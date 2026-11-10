//! Explicit, per-user installation of pinned portable conversion engines.
//! Downloaded programs are never executed here. A new directory is published
//! only after the complete archive and every extracted path have been checked.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

mod bundles;
mod http;
mod license_data;

const MARKER: &str = ".convert-here-runtime.json";
const MAX_FILES: usize = 4096;
const MAX_EXPANDED: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEngine {
    Ffmpeg,
    Poppler,
    Tesseract,
    Libheif,
    Whisper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeArtifact {
    pub name: String,
    pub version: String,
    pub download_bytes: u64,
    pub source_url: String,
    pub sha256: String,
    pub license: String,
}

#[derive(Debug)]
pub struct RuntimeManifest {
    pub name: &'static str,
    pub version: &'static str,
    pub download_bytes: u64,
    /// Materialized payload estimate; download archives and the transaction
    /// workspace temporarily require additional space during installation.
    pub expanded_bytes: u64,
    pub source_url: &'static str,
    pub sha256: &'static str,
    pub license: &'static str,
}

impl RuntimeManifest {
    pub fn license_label(&self) -> &'static str {
        match self.name {
            "FFmpeg" => crate::i18n::tr(self.license, "GPLv3; license and documentation included"),
            "Poppler" => crate::i18n::tr(
                self.license,
                "GPL and dependency licenses included in the archive",
            ),
            _ => self.license,
        }
    }
}

static FFMPEG: RuntimeManifest = RuntimeManifest {
    name: "FFmpeg",
    version: "9.0.2",
    download_bytes: 114_768_076,
    expanded_bytes: 329_221_735,
    source_url: "https://github.com/GyanD/codexffmpeg/releases/download/9.0.2/ffmpeg-9.0.2-essentials_build.zip",
    sha256: "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba",
    license: "GPLv3; licenza e documentazione incluse",
};
static POPPLER: RuntimeManifest = RuntimeManifest {
    name: "Poppler",
    version: "26.09.0",
    download_bytes: 43_709_956,
    expanded_bytes: 126_544_679,
    source_url: "https://github.com/oschwartz10612/poppler-windows/releases/download/v26.09.0-0/Release-26.09.0-0.zip",
    sha256: "7a6f256a0ddf7536182246a5733331bf4677cbcc34f4663774947ad34556c8d0",
    license: "GPL e licenze delle dipendenze incluse nell’archivio",
};
const FFMPEG_TOOLS: &[(&str, &str)] =
    &[("ffmpeg", "bin/ffmpeg.exe"), ("ffprobe", "bin/ffprobe.exe")];
const POPPLER_TOOLS: &[(&str, &str)] = &[
    ("pdftoppm", "Library/bin/pdftoppm.exe"),
    ("pdftotext", "Library/bin/pdftotext.exe"),
    ("pdfinfo", "Library/bin/pdfinfo.exe"),
];

impl RuntimeEngine {
    pub fn manifest(self) -> &'static RuntimeManifest {
        match self {
            Self::Ffmpeg => &FFMPEG,
            Self::Poppler => &POPPLER,
            Self::Tesseract | Self::Libheif | Self::Whisper => bundles::manifest(self),
        }
    }

    /// Every archive/model downloaded after the single consent prompt.
    pub fn artifacts(self) -> Vec<RuntimeArtifact> {
        match self {
            Self::Tesseract | Self::Libheif | Self::Whisper => bundles::artifacts(self),
            _ => {
                let manifest = self.manifest();
                vec![RuntimeArtifact {
                    name: manifest.name.into(),
                    version: manifest.version.into(),
                    download_bytes: manifest.download_bytes,
                    source_url: manifest.source_url.into(),
                    sha256: manifest.sha256.into(),
                    license: manifest.license.into(),
                }]
            }
        }
    }

    pub fn tool_paths(self, directory: &Path) -> Vec<(&'static str, PathBuf)> {
        self.spec()
            .required
            .iter()
            .map(|(name, relative)| (*name, joined(directory, relative)))
            .collect()
    }

    fn key(self) -> &'static str {
        match self {
            Self::Ffmpeg => "ffmpeg",
            Self::Poppler => "poppler",
            Self::Tesseract => "tesseract",
            Self::Libheif => "libheif",
            Self::Whisper => "whisper",
        }
    }

    fn spec(self) -> PackageSpec<'static> {
        if matches!(self, Self::Tesseract | Self::Libheif | Self::Whisper) {
            return bundles::spec(self);
        }
        let info = self.manifest();
        PackageSpec {
            engine: self,
            version: info.version,
            url: info.source_url,
            bytes: info.download_bytes,
            sha256: info.sha256,
            archive_root: match self {
                Self::Ffmpeg => "ffmpeg-9.0.2-essentials_build",
                Self::Poppler => "poppler-26.09.0",
                _ => unreachable!(),
            },
            required: match self {
                Self::Ffmpeg => FFMPEG_TOOLS,
                Self::Poppler => POPPLER_TOOLS,
                _ => unreachable!(),
            },
            license: match self {
                Self::Ffmpeg => "LICENSE",
                Self::Poppler => "share/poppler/COPYING",
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPhase {
    Connecting,
    Downloading,
    Verifying,
    Extracting,
    Installing,
    Complete,
}

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub engine: RuntimeEngine,
    pub phase: InstallPhase,
    pub fraction: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub message: String,
}

struct PackageSpec<'a> {
    engine: RuntimeEngine,
    version: &'a str,
    url: &'a str,
    bytes: u64,
    sha256: &'a str,
    archive_root: &'a str,
    required: &'a [(&'a str, &'a str)],
    license: &'a str,
}

fn update(
    spec: &PackageSpec<'_>,
    phase: InstallPhase,
    fraction: f32,
    bytes: u64,
    message: impl Into<String>,
) -> InstallProgress {
    InstallProgress {
        engine: spec.engine,
        phase,
        fraction,
        downloaded_bytes: bytes,
        total_bytes: spec.bytes,
        message: message.into(),
    }
}

/// Call from a worker only, after explicit user consent. Returns the engine's
/// directory; use `engine.tool_paths` to activate its main program and companions.
pub fn install(
    engine: RuntimeEngine,
    cancel: &AtomicBool,
    mut progress: impl FnMut(InstallProgress),
) -> Result<PathBuf> {
    let root = crate::tools::user_tools_dir().context(crate::i18n::tr(
        "Cartella personale dei motori non disponibile",
        "User engine folder unavailable",
    ))?;
    if matches!(
        engine,
        RuntimeEngine::Tesseract | RuntimeEngine::Libheif | RuntimeEngine::Whisper
    ) {
        return bundles::install(engine, &root, cancel, &mut progress);
    }
    install_with(
        &engine.spec(),
        &root,
        cancel,
        &mut progress,
        |path, spec, cancel, progress| download_with_retry(path, spec, cancel, progress),
    )
}

fn cancelled(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Relaxed),
        crate::trf!("Installazione annullata", "Installation cancelled")
    );
    Ok(())
}

fn joined(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_owned(), |path, component| path.join(component))
}

fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn ensure_directory(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute(),
        crate::trf!(
            "La cartella dei motori deve essere assoluta",
            "The engine folder path must be absolute"
        )
    );
    for component in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        match fs::symlink_metadata(component) {
            Ok(metadata) => ensure!(
                metadata.is_dir() && !is_link(&metadata),
                crate::trf!(
                    "Cartella non sicura o collegamento: {}",
                    "Unsafe folder or link: {}",
                    component.display()
                )
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(component)?
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn pin_directory(root: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // No FILE_SHARE_DELETE: the directory cannot be swapped while installing.
        options
            .share_mode(1 | 2)
            .custom_flags(0x02000000 | 0x00200000);
    }
    let file = options.open(root)?;
    ensure!(
        !is_link(&file.metadata()?),
        crate::trf!(
            "La cartella dei motori è un collegamento",
            "The engine folder is a link"
        )
    );
    Ok(file)
}

#[derive(Serialize, Deserialize)]
struct InstalledFile {
    bytes: u64,
    sha256: String,
}
#[derive(Serialize, Deserialize)]
struct InstallMarker {
    schema: u32,
    engine: String,
    version: String,
    source_url: String,
    sha256: String,
    files: BTreeMap<String, InstalledFile>,
}

fn hash_file(path: &Path, cancel: &AtomicBool) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        cancelled(cancel)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn installed_matches(
    destination: &Path,
    spec: &PackageSpec<'_>,
    cancel: &AtomicBool,
) -> Result<bool> {
    let marker_path = destination.join(MARKER);
    let Ok(metadata) = fs::symlink_metadata(&marker_path) else {
        return Ok(false);
    };
    if !metadata.is_file() || is_link(&metadata) || metadata.len() > 1_048_576 {
        return Ok(false);
    }
    let Ok(marker) = serde_json::from_slice::<InstallMarker>(&fs::read(&marker_path)?) else {
        return Ok(false);
    };
    if marker.schema != 1
        || marker.engine != spec.engine.key()
        || marker.version != spec.version
        || marker.source_url != spec.url
        || marker.sha256 != spec.sha256
    {
        return Ok(false);
    }
    if !spec
        .required
        .iter()
        .all(|(_, name)| marker.files.contains_key(*name))
        || !marker.files.contains_key(spec.license)
    {
        return Ok(false);
    }
    let mut pending = vec![destination.to_owned()];
    let mut found = 0;
    let mut entries = 0;
    while let Some(directory) = pending.pop() {
        cancelled(cancel)?;
        for entry in fs::read_dir(directory)? {
            cancelled(cancel)?;
            entries += 1;
            if entries > MAX_FILES {
                return Ok(false);
            }
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)?;
            if is_link(&metadata) {
                return Ok(false);
            }
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if path == marker_path {
                continue;
            }
            let relative = path
                .strip_prefix(destination)?
                .to_string_lossy()
                .replace('\\', "/");
            let Some(record) = marker.files.get(&relative) else {
                return Ok(false);
            };
            if !metadata.is_file()
                || metadata.len() != record.bytes
                || hash_file(&path, cancel)? != record.sha256
            {
                return Ok(false);
            }
            found += 1;
            if found > MAX_FILES {
                return Ok(false);
            }
        }
    }
    Ok(found == marker.files.len())
}

fn install_with(
    spec: &PackageSpec<'_>,
    root: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    mut download: impl FnMut(
        &Path,
        &PackageSpec<'_>,
        &AtomicBool,
        &mut dyn FnMut(InstallProgress),
    ) -> Result<()>,
) -> Result<PathBuf> {
    install_prepared(
        spec,
        root,
        cancel,
        progress,
        |workspace, prepared, cancel, progress| {
            let archive = workspace.join("download.zip");
            download(&archive, spec, cancel, progress)?;
            cancelled(cancel)?;
            let _archive_guard = pin_archive(&archive)?;
            progress(update(
                spec,
                InstallPhase::Verifying,
                0.72,
                spec.bytes,
                crate::i18n::tr(
                    "Verifica SHA-256 dell’archivio…",
                    "Verifying the archive's SHA-256…",
                ),
            ));
            ensure!(
                fs::metadata(&archive)?.len() == spec.bytes,
                crate::trf!(
                    "Dimensione dell’archivio diversa da quella prevista",
                    "The archive size does not match the expected size"
                )
            );
            ensure!(
                hash_file(&archive, cancel)? == spec.sha256,
                crate::trf!(
                    "Verifica SHA-256 non riuscita: nessun motore è stato installato",
                    "SHA-256 verification failed: no engine was installed"
                )
            );
            extract_archive(&archive, prepared, spec, cancel, progress)
        },
    )
}

fn install_prepared(
    spec: &PackageSpec<'_>,
    root: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    prepare: impl FnOnce(
        &Path,
        &Path,
        &AtomicBool,
        &mut dyn FnMut(InstallProgress),
    ) -> Result<BTreeMap<String, InstalledFile>>,
) -> Result<PathBuf> {
    cancelled(cancel)?;
    ensure_directory(root)?;
    let _directory_guard = pin_directory(root)?;
    let lock_path = root.join(format!(".install-{}.lock", spec.engine.key()));
    if let Ok(metadata) = fs::symlink_metadata(&lock_path) {
        ensure!(
            metadata.is_file() && !is_link(&metadata),
            crate::trf!(
                "Blocco di installazione non sicuro",
                "Unsafe installation lock"
            )
        );
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let lock = options.open(lock_path)?;
    ensure!(
        !is_link(&lock.metadata()?),
        crate::trf!(
            "Blocco di installazione non sicuro",
            "Unsafe installation lock"
        )
    );
    fs2::FileExt::try_lock_exclusive(&lock).context(crate::i18n::tr(
        "Un’altra installazione di questo motore è già in corso",
        "Another installation of this engine is already running",
    ))?;
    let destination = root.join(spec.engine.key());
    match fs::symlink_metadata(&destination) {
        Ok(metadata) => {
            ensure!(
                metadata.is_dir() && !is_link(&metadata),
                crate::trf!(
                    "La destinazione esistente non è una cartella sicura",
                    "The existing destination is not a safe folder"
                )
            );
            progress(update(
                spec,
                InstallPhase::Verifying,
                0.0,
                0,
                crate::i18n::tr(
                    "Verifica del motore già presente…",
                    "Verifying the existing engine…",
                ),
            ));
            if installed_matches(&destination, spec, cancel)? {
                progress(update(
                    spec,
                    InstallPhase::Complete,
                    1.0,
                    spec.bytes,
                    crate::i18n::tr(
                        "Motore già installato e verificato",
                        "Engine already installed and verified",
                    ),
                ));
                return Ok(destination);
            }
            bail!(crate::trf!(
                "La cartella {} esiste già e non corrisponde a questa installazione verificata. Non è stata modificata. Puoi selezionare il programma in Impostazioni → Motori.",
                "The folder {} already exists and does not match this verified installation. It was not modified. You can select the program in Settings → Engines.",
                destination.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let workspace = tempfile::Builder::new()
        .prefix(&format!(".install-{}-", spec.engine.key()))
        .tempdir_in(root)?;
    crate::platform::set_hidden(workspace.path())?;
    let prepared = workspace.path().join("prepared");
    fs::create_dir(&prepared)?;
    let files = prepare(workspace.path(), &prepared, cancel, progress)?;
    for required in spec
        .required
        .iter()
        .map(|(_, name)| *name)
        .chain([spec.license])
    {
        ensure!(
            files.get(required).is_some_and(|file| file.bytes > 0),
            crate::trf!(
                "Archivio incompleto: {required}",
                "Incomplete archive: {required}"
            )
        );
    }
    let marker = InstallMarker {
        schema: 1,
        engine: spec.engine.key().into(),
        version: spec.version.into(),
        source_url: spec.url.into(),
        sha256: spec.sha256.into(),
        files,
    };
    let mut marker_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(prepared.join(MARKER))?;
    serde_json::to_writer_pretty(&mut marker_file, &marker)?;
    marker_file.sync_all()?;
    drop(marker_file);
    cancelled(cancel)?;
    progress(update(
        spec,
        InstallPhase::Installing,
        0.99,
        spec.bytes,
        crate::i18n::tr(
            "Installazione nella cartella personale…",
            "Installing in the user folder…",
        ),
    ));
    publish_new_directory(&prepared, &destination)?;
    progress(update(
        spec,
        InstallPhase::Complete,
        1.0,
        spec.bytes,
        crate::i18n::tr(
            "Motore installato e verificato",
            "Engine installed and verified",
        ),
    ));
    Ok(destination)
}

fn pin_archive(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1).custom_flags(0x00200000);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file() && !is_link(&metadata),
        crate::trf!("Archivio non sicuro", "Unsafe archive")
    );
    Ok(file)
}

fn safe_relative(name: &str, expected_root: &str) -> Result<PathBuf> {
    ensure!(
        name.is_ascii() && !name.starts_with(['/', '\\']),
        crate::trf!("Nome ZIP non sicuro", "Unsafe ZIP entry name")
    );
    let normalized = name.replace('\\', "/");
    let parts: Vec<_> = normalized.trim_end_matches('/').split('/').collect();
    ensure!(
        parts.first() == Some(&expected_root),
        crate::trf!("Radice ZIP inattesa", "Unexpected ZIP root")
    );
    for component in &parts {
        ensure!(
            !component.is_empty()
                && *component != "."
                && *component != ".."
                && !component.ends_with([' ', '.'])
                && !component
                    .chars()
                    .any(|value| value.is_control() || "<>:\"|?*".contains(value)),
            crate::trf!("Percorso ZIP non sicuro", "Unsafe ZIP path")
        );
        let stem = component
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        ensure!(
            !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9')),
            crate::trf!(
                "Nome dispositivo Windows nel ZIP",
                "Windows device name in ZIP"
            )
        );
    }
    Ok(parts
        .into_iter()
        .skip(1)
        .fold(PathBuf::new(), |path, part| path.join(part)))
}

fn extract_archive(
    archive: &Path,
    destination: &Path,
    spec: &PackageSpec<'_>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
) -> Result<BTreeMap<String, InstalledFile>> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    let mut raw = File::open(archive)?;
    ensure!(
        zip.len() <= MAX_FILES,
        crate::trf!(
            "Archivio con troppi file",
            "The archive contains too many files"
        )
    );
    let mut seen = HashSet::new();
    let mut planned = Vec::with_capacity(zip.len());
    let mut total = 0u64;
    for index in 0..zip.len() {
        cancelled(cancel)?;
        let entry = zip.by_index(index)?;
        let relative = safe_relative(entry.name(), spec.archive_root)?;
        ensure!(
            !entry.encrypted() && !entry.is_symlink(),
            crate::trf!(
                "Collegamento o file cifrato nel ZIP",
                "Link or encrypted file in ZIP"
            )
        );
        if let Some(mode) = entry.unix_mode() {
            ensure!(
                matches!(mode & 0o170000, 0 | 0o040000 | 0o100000),
                crate::trf!(
                    "Tipo di file ZIP non consentito",
                    "Disallowed ZIP entry type"
                )
            );
        }
        raw.seek(SeekFrom::Start(
            entry
                .central_header_start()
                .checked_add(38)
                .context(crate::i18n::tr(
                    "Header ZIP non valido",
                    "Invalid ZIP header",
                ))?,
        ))?;
        let mut attributes = [0u8; 4];
        raw.read_exact(&mut attributes)?;
        ensure!(
            u32::from_le_bytes(attributes) & 0x400 == 0,
            crate::trf!("Reparse point nel ZIP", "Reparse point in ZIP")
        );
        if relative.as_os_str().is_empty() {
            ensure!(
                entry.is_dir(),
                crate::trf!("Radice ZIP non valida", "Invalid ZIP root")
            );
            continue;
        }
        ensure!(
            seen.insert(relative.to_string_lossy().to_ascii_lowercase()),
            crate::trf!("Percorso ZIP duplicato", "Duplicate ZIP path")
        );
        total = total.checked_add(entry.size()).context(crate::i18n::tr(
            "Dimensione ZIP eccessiva",
            "ZIP size limit exceeded",
        ))?;
        ensure!(
            total <= MAX_EXPANDED,
            crate::trf!(
                "Archivio espanso troppo grande",
                "The expanded archive is too large"
            )
        );
        planned.push((index, relative, entry.is_dir(), entry.size()));
    }
    let mut files = BTreeMap::new();
    let mut completed = 0u64;
    let mut buffer = [0u8; 65536];
    for (index, relative, directory, expected_size) in planned {
        cancelled(cancel)?;
        let target = destination.join(&relative);
        if directory {
            ensure_directory(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            ensure_directory(parent)?;
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        let mut entry = zip.by_index(index)?;
        let mut bytes = 0u64;
        let mut hash = Sha256::new();
        loop {
            cancelled(cancel)?;
            let count = entry.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            bytes += count as u64;
            ensure!(
                bytes <= expected_size,
                crate::trf!(
                    "File ZIP più grande di quanto dichiarato",
                    "ZIP entry exceeds its declared size"
                )
            );
            output.write_all(&buffer[..count])?;
            hash.update(&buffer[..count]);
            completed += count as u64;
            progress(update(
                spec,
                InstallPhase::Extracting,
                0.76 + 0.22 * completed as f32 / total.max(1) as f32,
                spec.bytes,
                crate::i18n::tr(
                    "Estrazione dei programmi e delle licenze…",
                    "Extracting programs and licenses…",
                ),
            ));
        }
        ensure!(
            bytes == expected_size,
            crate::trf!("File ZIP incompleto", "Incomplete ZIP entry")
        );
        files.insert(
            relative.to_string_lossy().replace('\\', "/"),
            InstalledFile {
                bytes,
                sha256: format!("{:x}", hash.finalize()),
            },
        );
    }
    Ok(files)
}

fn publish_new_directory(source: &Path, destination: &Path) -> Result<()> {
    ensure!(
        fs::symlink_metadata(destination)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        crate::trf!(
            "La destinazione è stata creata da un’altra operazione; nessun file sovrascritto",
            "Another operation created the destination; no files were overwritten"
        )
    );
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileW(from: *const u16, to: *const u16) -> i32;
        }
        let source: Vec<_> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<_> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        ensure!(
            unsafe { MoveFileW(source.as_ptr(), destination.as_ptr()) } != 0,
            crate::trf!(
                "Impossibile pubblicare il motore senza sovrascrivere file: {}",
                "Unable to publish the engine without overwriting files: {}",
                std::io::Error::last_os_error()
            )
        );
    }
    #[cfg(not(windows))]
    fs::rename(source, destination)?;
    Ok(())
}

fn download_with_retry(
    path: &Path,
    spec: &PackageSpec<'_>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
) -> Result<()> {
    retry_download(path, spec, cancel, progress, |path, spec, cancel, bytes| {
        http::download(spec.url, path, spec.bytes, cancel, bytes)
    })
}

fn retry_download(
    path: &Path,
    spec: &PackageSpec<'_>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    mut attempt_download: impl FnMut(
        &Path,
        &PackageSpec<'_>,
        &AtomicBool,
        &mut dyn FnMut(u64),
    ) -> Result<()>,
) -> Result<()> {
    let mut failure = None;
    for attempt in 1..=3 {
        cancelled(cancel)?;
        progress(update(
            spec,
            InstallPhase::Connecting,
            0.0,
            0,
            crate::trf!(
                "Connessione al distributore · tentativo {attempt}/3…",
                "Connecting to the distributor · attempt {attempt}/3…"
            ),
        ));
        let result = attempt_download(path, spec, cancel, &mut |bytes| {
            progress(update(
                spec,
                InstallPhase::Downloading,
                0.70 * bytes as f32 / spec.bytes.max(1) as f32,
                bytes,
                crate::i18n::tr(
                    "Download dell’archivio verificato…",
                    "Downloading the verified archive…",
                ),
            ));
        });
        match result {
            Ok(()) => return Ok(()),
            Err(error) => {
                cancelled(cancel)?;
                failure = Some(error);
            }
        }
        if attempt < 3 {
            for _ in 0..attempt * 2 {
                cancelled(cancel)?;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
    Err(failure.context(crate::i18n::tr(
        "Download non disponibile",
        "Download unavailable",
    ))?)
    .context(crate::i18n::tr(
        "Download non riuscito dopo tre tentativi",
        "Download failed after three attempts",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;

    fn archive(extra: Option<&str>) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for name in [
            "bin/ffmpeg.exe",
            "bin/ffprobe.exe",
            "LICENSE",
            "doc/README.txt",
        ]
        .into_iter()
        .chain(extra)
        {
            zip.start_file(format!("package/{name}"), options).unwrap();
            zip.write_all(b"Inert fixture, never executed; notices are preserved.")
                .unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    fn spec<'a>(bytes: &[u8], hash: &'a str) -> PackageSpec<'a> {
        PackageSpec {
            engine: RuntimeEngine::Ffmpeg,
            version: "fixture",
            url: "https://example.test/pinned.zip",
            bytes: bytes.len() as u64,
            sha256: hash,
            archive_root: "package",
            required: FFMPEG_TOOLS,
            license: "LICENSE",
        }
    }
    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    fn staged_directories(root: &Path) -> usize {
        fs::read_dir(root)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().unwrap().is_dir())
            .count()
    }
    fn install_fixture(root: &Path, bytes: &[u8], cancel: &AtomicBool) -> Result<PathBuf> {
        let hash = digest(bytes);
        install_with(
            &spec(bytes, &hash),
            root,
            cancel,
            &mut |_| {},
            |path, _, _, _| {
                fs::write(path, bytes)?;
                Ok(())
            },
        )
    }

    #[test]
    fn runtime_install_publishes_complete_upstream_files_and_is_idempotent() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("tools");
        let bytes = archive(None);
        let cancel = AtomicBool::new(false);
        let installed = install_fixture(&root, &bytes, &cancel).unwrap();
        assert_eq!(
            fs::read(installed.join("doc/README.txt")).unwrap(),
            b"Inert fixture, never executed; notices are preserved."
        );
        assert_eq!(
            RuntimeEngine::Ffmpeg.tool_paths(&installed)[1].1,
            installed.join("bin").join("ffprobe.exe")
        );
        let hash = digest(&bytes);
        let second = install_with(
            &spec(&bytes, &hash),
            &root,
            &cancel,
            &mut |_| {},
            |_, _, _, _| panic!("An identical verified install must not download again"),
        )
        .unwrap();
        assert_eq!(second, installed);
        assert_eq!(staged_directories(&root), 1);
    }

    #[test]
    fn runtime_install_hash_failure_rolls_back_and_a_fresh_retry_succeeds() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("tools");
        let bytes = archive(None);
        let cancel = AtomicBool::new(false);
        assert!(
            install_with(
                &spec(&bytes, "incorrect hash"),
                &root,
                &cancel,
                &mut |_| {},
                |path, _, _, _| {
                    fs::write(path, &bytes)?;
                    Ok(())
                }
            )
            .is_err()
        );
        assert!(!root.join("ffmpeg").exists());
        assert_eq!(staged_directories(&root), 0);
        assert!(install_fixture(&root, &bytes, &cancel).is_ok());
    }

    #[test]
    fn runtime_install_never_overwrites_unmanaged_or_modified_content() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("tools");
        let destination = root.join("ffmpeg");
        fs::create_dir_all(&destination).unwrap();
        let personal = destination.join("my-document.txt");
        fs::write(&personal, b"preserve me").unwrap();
        let bytes = archive(None);
        let cancel = AtomicBool::new(false);
        assert!(install_fixture(&root, &bytes, &cancel).is_err());
        assert_eq!(fs::read(&personal).unwrap(), b"preserve me");
        let other_root = temporary.path().join("other-tools");
        let installed = install_fixture(&other_root, &bytes, &cancel).unwrap();
        fs::write(
            installed.join("bin/ffmpeg.exe"),
            b"user-managed replacement",
        )
        .unwrap();
        assert!(install_fixture(&other_root, &bytes, &cancel).is_err());
        assert_eq!(
            fs::read(installed.join("bin/ffmpeg.exe")).unwrap(),
            b"user-managed replacement"
        );
    }

    #[test]
    fn runtime_install_cancellation_cleans_the_transaction_and_never_creates_output() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("tools");
        let bytes = archive(None);
        let cancel = AtomicBool::new(true);
        assert!(install_fixture(&root, &bytes, &cancel).is_err());
        assert!(!root.exists());
        cancel.store(false, Ordering::Relaxed);
        let hash = digest(&bytes);
        let result = install_with(
            &spec(&bytes, &hash),
            &root,
            &cancel,
            &mut |progress| {
                if progress.phase == InstallPhase::Extracting {
                    cancel.store(true, Ordering::Relaxed);
                }
            },
            |path, _, _, _| {
                fs::write(path, &bytes)?;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!root.join("ffmpeg").exists());
        assert_eq!(staged_directories(&root), 0);
        cancel.store(false, Ordering::Relaxed);
        assert!(install_fixture(&root, &bytes, &cancel).is_ok());
    }

    #[test]
    fn runtime_install_exclusive_lock_prevents_overlapping_transactions() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("tools");
        fs::create_dir(&root).unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(root.join(".install-ffmpeg.lock"))
            .unwrap();
        fs2::FileExt::lock_exclusive(&lock).unwrap();
        assert!(install_fixture(&root, &archive(None), &AtomicBool::new(false)).is_err());
        assert!(!root.join("ffmpeg").exists());
        drop(lock);
        assert!(install_fixture(&root, &archive(None), &AtomicBool::new(false)).is_ok());
    }

    #[test]
    fn runtime_install_rejects_zip_traversal_aliases_devices_and_duplicate_paths() {
        for name in [
            "../../escape.txt",
            "bin/../escape",
            "bin/file.exe:stream",
            "bin/NUL.txt",
            "bin/file. ",
            "BIN/FFMPEG.EXE",
        ] {
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("tools");
            assert!(
                install_fixture(&root, &archive(Some(name)), &AtomicBool::new(false)).is_err(),
                "accepted {name}"
            );
            assert!(!root.join("ffmpeg").exists());
            assert_eq!(staged_directories(&root), 0);
            assert!(!temporary.path().join("escape.txt").exists());
        }
        assert!(safe_relative("/package/file", "package").is_err());
        assert!(safe_relative("other-root/file", "package").is_err());
    }

    #[test]
    fn runtime_install_rejects_zip_links_reparse_attributes_and_expansion_bombs() {
        let mut linked = zip::ZipWriter::new(Cursor::new(Vec::new()));
        linked
            .add_symlink(
                "package/link",
                "../../outside",
                SimpleFileOptions::default(),
            )
            .unwrap();
        let bytes = linked.finish().unwrap().into_inner();
        let temporary = tempfile::tempdir().unwrap();
        assert!(
            install_fixture(
                &temporary.path().join("link-tools"),
                &bytes,
                &AtomicBool::new(false)
            )
            .is_err()
        );
        for (offset, value) in [(38, 0x400u32), (24, MAX_EXPANDED as u32 + 1)] {
            let mut bytes = archive(None);
            let central = bytes
                .windows(4)
                .position(|window| window == b"PK\x01\x02")
                .unwrap();
            bytes[central + offset..central + offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(
                install_fixture(
                    &temporary.path().join(format!("tools-{offset}")),
                    &bytes,
                    &AtomicBool::new(false)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn runtime_install_retry_restarts_a_partial_download_without_reusing_bytes() {
        let temporary = tempfile::tempdir().unwrap();
        let bytes = archive(None);
        let hash = digest(&bytes);
        let package = spec(&bytes, &hash);
        let root = temporary.path().join("tools");
        let cancel = AtomicBool::new(false);
        let mut attempts = 0;
        let result = install_with(
            &package,
            &root,
            &cancel,
            &mut |_| {},
            |path, spec, cancel, progress| {
                retry_download(path, spec, cancel, progress, |path, _, _, _| {
                    attempts += 1;
                    if attempts == 1 {
                        fs::write(path, b"partial")?;
                        bail!("Transient fixture failure");
                    }
                    fs::write(path, &bytes)?;
                    Ok(())
                })
            },
        );
        assert!(result.is_ok());
        assert_eq!(attempts, 2);
    }

    #[test]
    #[ignore = "Validates cached pinned upstream ZIPs in a temporary directory; never executes them"]
    fn runtime_install_cached_upstream_archives_preserve_tools_and_licenses() {
        for engine in [RuntimeEngine::Ffmpeg, RuntimeEngine::Poppler] {
            let source = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(".tools/downloads")
                .join(format!("{}.zip", engine.key()));
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("tools");
            let result = install_with(
                &engine.spec(),
                &root,
                &AtomicBool::new(false),
                &mut |_| {},
                |path, _, _, _| {
                    fs::copy(&source, path)?;
                    Ok(())
                },
            )
            .unwrap();
            assert!(
                engine
                    .tool_paths(&result)
                    .iter()
                    .all(|(_, path)| path.is_file())
            );
            assert!(joined(&result, engine.spec().license).is_file());
        }
    }
}

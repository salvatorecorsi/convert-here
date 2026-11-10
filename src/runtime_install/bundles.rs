//! Fixed portable distributions, assembled without running their setup scripts.
use super::*;
use std::sync::OnceLock;

const MAX_BUNDLE_DECLARED: u64 = 1024 * 1024 * 1024;

#[derive(Deserialize)]
struct Bundle {
    name: String,
    version: String,
    source_url: String,
    license: String,
    license_file: String,
    fingerprint: String,
    required: Vec<String>,
    artifacts: Vec<Artifact>,
    #[serde(default)]
    licenses: Vec<LicenseSupplement>,
}

#[derive(Clone, Deserialize, Serialize)]
struct LicenseSupplement {
    filename: String,
    destination: String,
    bytes: u64,
    sha256: String,
    #[serde(flatten)]
    provenance: serde_json::Map<String, serde_json::Value>,
}
#[derive(Deserialize)]
struct Artifact {
    #[serde(flatten)]
    info: RuntimeArtifact,
    expanded_bytes: u64,
    format: String,
    archive_root: String,
    destination: String,
    #[serde(default)]
    install_files: Option<Vec<String>>,
}
fn catalog() -> &'static BTreeMap<String, Bundle> {
    static DATA: OnceLock<BTreeMap<String, Bundle>> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("portable-packages.json"))
            .expect("compiled portable runtime catalog")
    })
}
fn bundle(engine: RuntimeEngine) -> &'static Bundle {
    &catalog()[engine.key()]
}
pub(super) fn manifest(engine: RuntimeEngine) -> &'static RuntimeManifest {
    static DATA: OnceLock<BTreeMap<String, RuntimeManifest>> = OnceLock::new();
    &DATA.get_or_init(|| {
        catalog()
            .iter()
            .map(|(key, item)| {
                (
                    key.clone(),
                    RuntimeManifest {
                        name: &item.name,
                        version: &item.version,
                        source_url: &item.source_url,
                        license: &item.license,
                        sha256: &item.fingerprint,
                        download_bytes: item.artifacts.iter().map(|a| a.info.download_bytes).sum(),
                        expanded_bytes: item
                            .artifacts
                            .iter()
                            .map(|a| a.expanded_bytes)
                            .sum::<u64>()
                            + item.licenses.iter().map(|file| file.bytes).sum::<u64>(),
                    },
                )
            })
            .collect()
    })[engine.key()]
}
pub(super) fn artifacts(engine: RuntimeEngine) -> Vec<RuntimeArtifact> {
    bundle(engine)
        .artifacts
        .iter()
        .map(|a| a.info.clone())
        .collect()
}
pub(super) fn spec(engine: RuntimeEngine) -> PackageSpec<'static> {
    let item = bundle(engine);
    let info = manifest(engine);
    let required: &[(&str, &str)] = match engine {
        RuntimeEngine::Tesseract => &[("tesseract", "bin/tesseract.exe")],
        RuntimeEngine::Libheif => &[
            ("heif-enc", "bin/heif-enc.exe"),
            ("heif-dec", "bin/heif-dec.exe"),
        ],
        RuntimeEngine::Whisper => &[
            ("whisper-cli", "bin/whisper-cli.exe"),
            ("whisper-model", "models/ggml-tiny.bin"),
        ],
        _ => unreachable!(),
    };
    PackageSpec {
        engine,
        version: info.version,
        url: info.source_url,
        bytes: info.download_bytes,
        sha256: info.sha256,
        archive_root: "",
        required,
        license: &item.license_file,
    }
}
pub(super) fn install(
    engine: RuntimeEngine,
    root: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
) -> Result<PathBuf> {
    install_prepared(
        &spec(engine),
        root,
        cancel,
        progress,
        |workspace, prepared, cancel, progress| {
            prepare_bundle(
                bundle(engine),
                &spec(engine),
                workspace,
                prepared,
                cancel,
                progress,
                |path, component, cancel, progress| {
                    download_with_retry(path, component, cancel, progress)
                },
            )
        },
    )
}
#[allow(clippy::too_many_arguments)]
fn prepare_bundle(
    item: &Bundle,
    spec: &PackageSpec<'_>,
    workspace: &Path,
    destination: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    mut download: impl FnMut(
        &Path,
        &PackageSpec<'_>,
        &AtomicBool,
        &mut dyn FnMut(InstallProgress),
    ) -> Result<()>,
) -> Result<BTreeMap<String, InstalledFile>> {
    ensure!(
        !item.artifacts.is_empty() && item.artifacts.len() <= 128,
        "Invalid bundle component count"
    );
    let mut archives = Vec::new();
    let mut guards = Vec::new();
    let mut completed = 0u64;
    for (index, artifact) in item.artifacts.iter().enumerate() {
        cancelled(cancel)?;
        let path = workspace.join(format!("component-{index}"));
        let component = PackageSpec {
            engine: spec.engine,
            version: &artifact.info.version,
            url: &artifact.info.source_url,
            bytes: artifact.info.download_bytes,
            sha256: &artifact.info.sha256,
            archive_root: &artifact.archive_root,
            required: &[],
            license: "",
        };
        download(&path, &component, cancel, &mut |event| {
            let bytes = completed + event.downloaded_bytes.min(component.bytes);
            progress(update(
                spec,
                event.phase,
                0.7 * bytes as f32 / spec.bytes.max(1) as f32,
                bytes,
                format!("{}: {}", artifact.info.name, event.message),
            ));
        })?;
        guards.push(pin_archive(&path)?);
        ensure!(
            fs::metadata(&path)?.len() == component.bytes,
            "Unexpected component size: {}",
            artifact.info.name
        );
        ensure!(
            hash_file(&path, cancel)? == component.sha256,
            "SHA-256 verification failed: {}",
            artifact.info.name
        );
        completed += component.bytes;
        progress(update(
            spec,
            InstallPhase::Verifying,
            0.7 * completed as f32 / spec.bytes.max(1) as f32,
            completed,
            format!("SHA-256: {}", artifact.info.name),
        ));
        archives.push(path);
    }
    let mut files = BTreeMap::new();
    let mut seen = HashSet::new();
    let mut expanded = 0u64;
    for (index, (artifact, path)) in item.artifacts.iter().zip(&archives).enumerate() {
        cancelled(cancel)?;
        match artifact.format.as_str() {
            "tar_zstd" => extract_tar(
                path,
                destination,
                artifact,
                cancel,
                &mut files,
                &mut seen,
                &mut expanded,
            )?,
            "zip" => extract_zip(
                path,
                destination,
                artifact,
                cancel,
                &mut files,
                &mut seen,
                &mut expanded,
            )?,
            "raw" => {
                add_size(&mut expanded, artifact.info.download_bytes)?;
                let relative = checked_relative(&artifact.destination)?;
                copy_entry(
                    &mut File::open(path)?,
                    destination,
                    &relative,
                    artifact.info.download_bytes,
                    cancel,
                    &mut files,
                    &mut seen,
                )?;
            }
            _ => bail!("Unknown bundle archive format"),
        }
        progress(update(
            spec,
            InstallPhase::Extracting,
            0.72 + 0.26 * (index + 1) as f32 / archives.len() as f32,
            spec.bytes,
            format!("{} ({}/{})", artifact.info.name, index + 1, archives.len()),
        ));
    }
    for license in &item.licenses {
        let bytes = super::license_data::bytes(&license.filename)
            .context("Missing embedded upstream license")?;
        ensure!(
            bytes.len() as u64 == license.bytes
                && format!("{:x}", Sha256::digest(bytes)) == license.sha256,
            "Embedded license hash mismatch: {}",
            license.filename
        );
        copy_entry(
            &mut &bytes[..],
            destination,
            &checked_relative(&license.destination)?,
            license.bytes,
            cancel,
            &mut files,
            &mut seen,
        )?;
    }
    for path in item.required.iter().chain([&item.license_file]) {
        ensure!(
            files.get(path).is_some_and(|file| file.bytes > 0),
            "Incomplete portable runtime: {path}"
        );
    }
    let sources =
        serde_json::to_vec_pretty(&item.artifacts.iter().map(|a| &a.info).collect::<Vec<_>>())?;
    copy_entry(
        &mut sources.as_slice(),
        destination,
        Path::new("RUNTIME-SOURCES.json"),
        sources.len() as u64,
        cancel,
        &mut files,
        &mut seen,
    )?;
    let licenses = serde_json::to_vec_pretty(&item.licenses)?;
    copy_entry(
        &mut licenses.as_slice(),
        destination,
        Path::new("LICENSE-SOURCES.json"),
        licenses.len() as u64,
        cancel,
        &mut files,
        &mut seen,
    )?;
    Ok(files)
}
fn checked_relative(name: &str) -> Result<PathBuf> {
    ensure!(name.len() <= 1024, "Archive path is too long");
    let relative = safe_relative(&format!("bundle/{name}"), "bundle")?;
    ensure!(!relative.as_os_str().is_empty(), "Empty bundle destination");
    Ok(relative)
}
fn selected_path(relative: &Path, artifact: &Artifact) -> Result<Option<PathBuf>> {
    let name = relative.to_string_lossy().replace('\\', "/");
    if let Some(allowed) = &artifact.install_files
        && !allowed.iter().any(|entry| entry == &name)
    {
        return Ok(None);
    }
    let lower = name.to_ascii_lowercase();
    let file = lower.rsplit('/').next().unwrap_or_default();
    let keep = lower.starts_with("share/licenses/")
        || lower.starts_with("share/doc/")
        || lower.starts_with("share/icu/") && matches!(file, "license" | "license.txt")
        || lower.starts_with("share/tessdata/")
        || file.ends_with(".dll")
            && !matches!(
                file,
                "libcommon_training.dll" | "libpango_training.dll" | "libunicharset_training.dll"
            )
        || matches!(
            file,
            "tesseract.exe" | "heif-enc.exe" | "heif-dec.exe" | "whisper-cli.exe"
        );
    if !keep {
        return Ok(None);
    }
    let name = if let Some(data) = name.strip_prefix("share/tessdata/") {
        format!("bin/tessdata/{data}")
    } else if artifact.destination.is_empty() {
        name
    } else {
        format!("{}/{name}", artifact.destination)
    };
    Ok(Some(checked_relative(&name)?))
}
fn add_size(total: &mut u64, bytes: u64) -> Result<()> {
    *total = total
        .checked_add(bytes)
        .context("Expanded archive size overflow")?;
    ensure!(
        *total <= MAX_BUNDLE_DECLARED,
        "Expanded runtime exceeds size limit"
    );
    Ok(())
}
fn copy_entry(
    input: &mut dyn Read,
    destination: &Path,
    relative: &Path,
    expected: u64,
    cancel: &AtomicBool,
    files: &mut BTreeMap<String, InstalledFile>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    let name = relative.to_string_lossy().replace('\\', "/");
    ensure!(
        seen.insert(name.to_ascii_lowercase()) && seen.len() <= MAX_FILES,
        "Duplicate or excessive runtime files: {name}"
    );
    let target = destination.join(relative);
    ensure_directory(target.parent().context("Missing file parent")?)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let mut bytes = 0u64;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        cancelled(cancel)?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        ensure!(bytes <= expected, "Archive entry exceeds declared size");
        output.write_all(&buffer[..count])?;
        hash.update(&buffer[..count]);
    }
    ensure!(bytes == expected, "Incomplete runtime file: {name}");
    files.insert(
        name,
        InstalledFile {
            bytes,
            sha256: format!("{:x}", hash.finalize()),
        },
    );
    Ok(())
}
fn tar_reader(path: &Path) -> Result<tar::Archive<impl Read>> {
    Ok(tar::Archive::new(
        zstd::stream::read::Decoder::new(File::open(path)?)?.take(MAX_BUNDLE_DECLARED + 1),
    ))
}
#[allow(clippy::too_many_arguments)]
fn extract_tar(
    path: &Path,
    destination: &Path,
    artifact: &Artifact,
    cancel: &AtomicBool,
    files: &mut BTreeMap<String, InstalledFile>,
    seen: &mut HashSet<String>,
    expanded: &mut u64,
) -> Result<()> {
    let mut plan = BTreeMap::new();
    let mut paths = HashSet::new();
    for (index, entry) in tar_reader(path)?.entries()?.enumerate() {
        cancelled(cancel)?;
        ensure!(index < 16384, "Too many TAR entries");
        let entry = entry?;
        let kind = entry.header().entry_type();
        let size = entry.size();
        add_size(expanded, size)?;
        let name = std::str::from_utf8(&entry.path_bytes())?.to_owned();
        if kind.is_hard_link() {
            // MSYS2 aliases unused command-line programs with hardlinks. They
            // are never materialized; only excluded, safe-root aliases may be skipped.
            let relative = safe_relative(&name, &artifact.archive_root)?;
            let link = entry.link_name()?.context("Missing TAR hardlink target")?;
            safe_relative(&link.to_string_lossy(), &artifact.archive_root)?;
            ensure!(
                size == 0
                    && artifact
                        .install_files
                        .as_ref()
                        .is_some_and(|allowed| !allowed
                            .iter()
                            .any(|path| path == &relative.to_string_lossy().replace('\\', "/"))),
                "Selected or unsafe TAR hardlink"
            );
            continue;
        }
        ensure!(
            kind.is_file() || kind.is_dir(),
            "Links or special files are forbidden in TAR: {name}"
        );
        if matches!(
            name.as_str(),
            ".PKGINFO" | ".BUILDINFO" | ".MTREE" | ".INSTALL"
        ) {
            continue;
        }
        let relative = safe_relative(&name, &artifact.archive_root)?;
        if relative.as_os_str().is_empty() {
            ensure!(kind.is_dir(), "Invalid archive root");
            continue;
        }
        ensure!(
            paths.insert(relative.to_string_lossy().to_ascii_lowercase()),
            "Duplicate TAR path"
        );
        if kind.is_dir() {
            ensure!(size == 0, "Invalid TAR directory size");
            continue;
        }
        if let Some(relative) = selected_path(&relative, artifact)? {
            plan.insert(index, (relative, size));
        }
    }
    for (index, entry) in tar_reader(path)?.entries()?.enumerate() {
        cancelled(cancel)?;
        let mut entry = entry?;
        if let Some((relative, size)) = plan.get(&index) {
            copy_entry(
                &mut entry,
                destination,
                relative,
                *size,
                cancel,
                files,
                seen,
            )?;
        }
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn extract_zip(
    path: &Path,
    destination: &Path,
    artifact: &Artifact,
    cancel: &AtomicBool,
    files: &mut BTreeMap<String, InstalledFile>,
    seen: &mut HashSet<String>,
    expanded: &mut u64,
) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(path)?)?;
    let mut raw = File::open(path)?;
    ensure!(zip.len() <= MAX_FILES, "Too many ZIP entries");
    let mut paths = HashSet::new();
    let mut plan = Vec::new();
    for index in 0..zip.len() {
        cancelled(cancel)?;
        let entry = zip.by_index(index)?;
        ensure!(
            !entry.encrypted() && !entry.is_symlink(),
            "Encrypted files or links in ZIP"
        );
        if let Some(mode) = entry.unix_mode() {
            ensure!(
                matches!(mode & 0o170000, 0 | 0o040000 | 0o100000),
                "Special ZIP file"
            );
        }
        raw.seek(SeekFrom::Start(
            entry
                .central_header_start()
                .checked_add(38)
                .context("Invalid ZIP header")?,
        ))?;
        let mut attributes = [0u8; 4];
        raw.read_exact(&mut attributes)?;
        ensure!(
            u32::from_le_bytes(attributes) & 0x400 == 0,
            "Reparse point in ZIP"
        );
        let relative = safe_relative(entry.name(), &artifact.archive_root)?;
        if relative.as_os_str().is_empty() {
            ensure!(entry.is_dir(), "Invalid ZIP root");
            continue;
        }
        ensure!(
            paths.insert(relative.to_string_lossy().to_ascii_lowercase()),
            "Duplicate ZIP path"
        );
        add_size(expanded, entry.size())?;
        if entry.is_dir() {
            ensure!(entry.size() == 0, "Invalid ZIP directory size");
            continue;
        }
        if let Some(relative) = selected_path(&relative, artifact)? {
            plan.push((index, relative, entry.size()));
        }
    }
    for (index, relative, size) in plan {
        copy_entry(
            &mut zip.by_index(index)?,
            destination,
            &relative,
            size,
            cancel,
            files,
            seen,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    fn packed(entries: &[(&str, &[u8])], link: bool) -> Vec<u8> {
        let encoder = zstd::Encoder::new(Vec::new(), 1).unwrap();
        let mut tar = tar::Builder::new(encoder);
        for (name, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            if link {
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name("other.dll").unwrap();
            }
            header.set_cksum();
            tar.append(&header, *bytes).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }
    fn fixture(archive: &[u8], model: &[u8]) -> Bundle {
        let artifact = |name: &str, bytes: &[u8], format: &str, target: &str| Artifact {
            info: RuntimeArtifact {
                name: name.into(),
                version: "fixture".into(),
                download_bytes: bytes.len() as u64,
                source_url: format!("https://example.invalid/{name}"),
                sha256: digest(bytes),
                license: "Fixture".into(),
            },
            expanded_bytes: 1000,
            format: format.into(),
            archive_root: "ucrt64".into(),
            destination: target.into(),
            install_files: None,
        };
        Bundle {
            name: "Fixture".into(),
            version: "1".into(),
            source_url: "https://example.invalid/bundle".into(),
            license: "Fixture".into(),
            license_file: "share/licenses/runtime/LICENSE".into(),
            fingerprint: "fixture-fingerprint".into(),
            required: vec![
                "bin/tesseract.exe".into(),
                "bin/helper.dll".into(),
                "bin/tessdata/eng.traineddata".into(),
            ],
            licenses: Vec::new(),
            artifacts: vec![
                artifact("package", archive, "tar_zstd", ""),
                artifact("model", model, "raw", "bin/tessdata/eng.traineddata"),
            ],
        }
    }
    fn fixture_install(
        root: &Path,
        item: &Bundle,
        bytes: &[&[u8]],
        cancel: &AtomicBool,
    ) -> Result<PathBuf> {
        let required = [("tesseract", "bin/tesseract.exe")];
        let spec = PackageSpec {
            engine: RuntimeEngine::Tesseract,
            version: &item.version,
            url: &item.source_url,
            sha256: &item.fingerprint,
            bytes: item.artifacts.iter().map(|a| a.info.download_bytes).sum(),
            archive_root: "",
            required: &required,
            license: &item.license_file,
        };
        let mut index = 0;
        install_prepared(
            &spec,
            root,
            cancel,
            &mut |_| {},
            |workspace, dest, cancel, progress| {
                prepare_bundle(
                    item,
                    &spec,
                    workspace,
                    dest,
                    cancel,
                    progress,
                    |path, _, cancel, _| {
                        cancelled(cancel)?;
                        fs::write(path, bytes[index])?;
                        index += 1;
                        Ok(())
                    },
                )
            },
        )
    }
    #[test]
    fn bundle_install_keeps_dll_models_notices_and_is_idempotent_without_running_programs() {
        let archive = packed(
            &[
                ("ucrt64/bin/tesseract.exe", b"inert exe"),
                ("ucrt64/bin/helper.dll", b"inert dll"),
                ("ucrt64/bin/not-needed.exe", b"not installed"),
                (
                    "ucrt64/share/licenses/runtime/LICENSE",
                    b"unaltered copyright",
                ),
                ("ucrt64/share/icu/78.3/LICENSE", b"ICU copyright"),
                ("ucrt64/share/tessdata/configs/default", b"OCR config"),
            ],
            false,
        );
        let model = b"model data, never executed";
        let item = fixture(&archive, model);
        let root = tempfile::tempdir().unwrap();
        let cancel = AtomicBool::new(false);
        let installed = fixture_install(root.path(), &item, &[&archive, model], &cancel).unwrap();
        assert_eq!(
            fs::read(installed.join("bin/tessdata/eng.traineddata")).unwrap(),
            model
        );
        assert_eq!(
            fs::read(installed.join("bin/tessdata/configs/default")).unwrap(),
            b"OCR config"
        );
        assert_eq!(
            fs::read(installed.join("share/icu/78.3/LICENSE")).unwrap(),
            b"ICU copyright"
        );
        assert!(!installed.join("bin/not-needed.exe").exists());
        assert!(installed.join("RUNTIME-SOURCES.json").is_file());
        assert_eq!(
            fixture_install(root.path(), &item, &[], &cancel).unwrap(),
            installed
        );
        fs::write(installed.join("bin/helper.dll"), b"modified").unwrap();
        assert!(fixture_install(root.path(), &item, &[], &cancel).is_err());
        assert_eq!(
            fs::read(installed.join("bin/helper.dll")).unwrap(),
            b"modified"
        );
    }
    #[test]
    fn bundle_rejects_bad_later_hash_or_links_and_rolls_back_every_component() {
        let good = packed(
            &[
                ("ucrt64/bin/tesseract.exe", b"exe"),
                ("ucrt64/bin/helper.dll", b"dll"),
                ("ucrt64/share/licenses/runtime/LICENSE", b"license"),
            ],
            false,
        );
        let model = b"model";
        let mut item = fixture(&good, model);
        item.artifacts[1].info.sha256 = "invalid".into();
        let root = tempfile::tempdir().unwrap();
        let cancel = AtomicBool::new(false);
        assert!(fixture_install(root.path(), &item, &[&good, model], &cancel).is_err());
        assert!(!root.path().join("tesseract").exists());
        assert!(
            !fs::read_dir(root.path())
                .unwrap()
                .any(|e| e.unwrap().file_type().unwrap().is_dir())
        );
        let linked = packed(&[("ucrt64/bin/helper.dll", b"")], true);
        let item = fixture(&linked, model);
        assert!(fixture_install(root.path(), &item, &[&linked, model], &cancel).is_err());
        assert!(!root.path().join("tesseract").exists());
        cancel.store(true, Ordering::Relaxed);
        assert!(fixture_install(root.path(), &fixture(&good, model), &[], &cancel).is_err());
    }
    #[test]
    fn bundle_rejects_traversal_device_names_duplicates_and_expanded_limit() {
        for name in [
            "../outside",
            "bin/NUL.dll",
            "bin/a:stream",
            "bin/file. ",
            "/absolute",
        ] {
            assert!(checked_relative(name).is_err(), "{name}");
        }
        let archive = packed(
            &[
                ("ucrt64/bin/tesseract.exe", b"one"),
                ("ucrt64/bin/TESSERACT.EXE", b"two"),
            ],
            false,
        );
        let root = tempfile::tempdir().unwrap();
        assert!(
            fixture_install(
                root.path(),
                &fixture(&archive, b"model"),
                &[&archive, b"model"],
                &AtomicBool::new(false)
            )
            .is_err()
        );
        let mut expanded = MAX_BUNDLE_DECLARED;
        assert!(add_size(&mut expanded, 1).is_err());
    }
    #[test]
    fn tar_hardlink_aliases_are_only_skipped_when_excluded_and_never_materialized() {
        let encoder = zstd::Encoder::new(Vec::new(), 1).unwrap();
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_path("ucrt64/bin/unused.exe").unwrap();
        header.set_entry_type(tar::EntryType::Link);
        header
            .set_link_name("ucrt64/bin/another-unused.exe")
            .unwrap();
        header.set_mode(0o644);
        header.set_size(0);
        header.set_cksum();
        archive.append(&header, &b""[..]).unwrap();
        let bytes = archive.into_inner().unwrap().finish().unwrap();
        let mut item = fixture(&bytes, b"model");
        item.artifacts[0].install_files = Some(vec!["bin/tesseract.exe".into()]);
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("package.zst");
        fs::write(&path, bytes).unwrap();
        let output = temp.path().join("extracted");
        fs::create_dir(&output).unwrap();
        let mut files = BTreeMap::new();
        extract_tar(
            &path,
            &output,
            &item.artifacts[0],
            &AtomicBool::new(false),
            &mut files,
            &mut HashSet::new(),
            &mut 0,
        )
        .unwrap();
        assert!(files.is_empty() && fs::read_dir(&output).unwrap().next().is_none());
        item.artifacts[0].install_files = Some(vec!["bin/unused.exe".into()]);
        assert!(
            extract_tar(
                &path,
                &output,
                &item.artifacts[0],
                &AtomicBool::new(false),
                &mut files,
                &mut HashSet::new(),
                &mut 0
            )
            .is_err()
        );
    }
    #[test]
    fn embedded_notices_are_verified_and_a_bad_pin_never_publishes() {
        let archive = packed(
            &[
                ("ucrt64/bin/tesseract.exe", b"exe"),
                ("ucrt64/bin/helper.dll", b"dll"),
                ("ucrt64/share/licenses/runtime/LICENSE", b"copyright"),
            ],
            false,
        );
        let mut item = fixture(&archive, b"model");
        item.licenses
            .push(bundle(RuntimeEngine::Tesseract).licenses[0].clone());
        let root = tempfile::tempdir().unwrap();
        let installed = fixture_install(
            root.path(),
            &item,
            &[&archive, b"model"],
            &AtomicBool::new(false),
        )
        .unwrap();
        let license = &item.licenses[0];
        assert_eq!(
            hash_file(
                &joined(&installed, &license.destination),
                &AtomicBool::new(false)
            )
            .unwrap(),
            license.sha256
        );
        assert!(installed.join("LICENSE-SOURCES.json").is_file());
        item.licenses[0].sha256 = "tampered pin".into();
        let second = tempfile::tempdir().unwrap();
        assert!(
            fixture_install(
                second.path(),
                &item,
                &[&archive, b"model"],
                &AtomicBool::new(false)
            )
            .is_err()
        );
        assert!(!second.path().join("tesseract").exists());
        item.licenses[0].filename = "missing-notice".into();
        assert!(
            fixture_install(
                second.path(),
                &item,
                &[&archive, b"model"],
                &AtomicBool::new(false)
            )
            .is_err()
        );
    }
    #[test]
    fn compiled_bundles_have_exact_public_pins_unique_destinations_and_supported_formats() {
        for engine in [
            RuntimeEngine::Tesseract,
            RuntimeEngine::Libheif,
            RuntimeEngine::Whisper,
        ] {
            let item = bundle(engine);
            let info = manifest(engine);
            assert!(!item.required.is_empty());
            assert!(info.download_bytes > 0 && info.expanded_bytes < MAX_EXPANDED);
            assert_eq!(info.sha256.len(), 64);
            let mut sources = HashSet::new();
            for artifact in &item.artifacts {
                assert!(artifact.info.source_url.starts_with("https://"));
                assert!(sources.insert(&artifact.info.source_url));
                assert!(
                    artifact.info.sha256.len() == 64
                        && artifact.info.sha256.chars().all(|c| c.is_ascii_hexdigit())
                );
                assert!(matches!(
                    artifact.format.as_str(),
                    "tar_zstd" | "zip" | "raw"
                ));
                if artifact.format == "raw" {
                    checked_relative(&artifact.destination).unwrap();
                }
            }
            assert_eq!(artifacts(engine).len(), item.artifacts.len());
        }
    }
}

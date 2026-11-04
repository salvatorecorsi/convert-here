//! Persistent per-user quick-conversion queue. Explorer previews exist before any encoder starts.
use crate::{
    engine::{self, PreparedConversion},
    model::ConversionSettings,
    pending::PendingLink,
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const COMPLETED_RETENTION_MS: u64 = 7 * 24 * 60 * 60 * 1_000;
const CANCELLED_RETENTION_MS: u64 = COMPLETED_RETENTION_MS;
const FAILED_RETENTION_MS: u64 = 30 * 24 * 60 * 60 * 1_000;
const MAX_TERMINAL_JOBS: usize = 300;
const PROGRESS_PREVIEW_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}
impl JobState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueJob {
    pub id: String,
    pub input: PathBuf,
    pub format: String,
    #[serde(default)]
    pub outputs: Vec<PathBuf>,
    pub state: JobState,
    #[serde(default)]
    pub progress: f32,
    #[serde(default)]
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredJob {
    #[serde(flatten)]
    view: QueueJob,
    plan: PreparedConversion,
    #[serde(default)]
    tools: ToolPaths,
    #[serde(default)]
    previews: Vec<PendingLink>,
    #[serde(default)]
    batch_id: String,
    #[serde(default)]
    submitted_at: u64,
    #[serde(default)]
    updated_at: u64,
}

pub struct Submission {
    pub jobs: Vec<QueueJob>,
    pub errors: Vec<String>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn queue_root() -> Result<PathBuf> {
    let path = if let Some(value) = std::env::var_os("THE_CONVERTER_QUEUE_DIR") {
        PathBuf::from(value)
    } else {
        directories::ProjectDirs::from("com", "SalvatoreCorsi", "TheConverter")
            .context("Cartella della coda utente non disponibile")?
            .data_local_dir()
            .join("queue")
    };
    fs::create_dir_all(&path).context("Impossibile creare la coda di conversione")?;
    Ok(path)
}
pub fn last_report_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("THE_CONVERTER_QUEUE_DIR") {
        return Some(PathBuf::from(root).join("report-last.json"));
    }
    directories::ProjectDirs::from("com", "SalvatoreCorsi", "TheConverter")
        .map(|dirs| dirs.config_dir().join("report-last.json"))
}
fn valid_id(id: &str) -> bool {
    id.starts_with("job-")
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn job_dir(root: &Path, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        bail!("Identificatore della conversione non valido")
    }
    Ok(root.join(id))
}
fn open_lock(root: &Path, name: &str) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(name))?)
}
fn dispatch_lock(root: &Path) -> Result<File> {
    let file = open_lock(root, "dispatch.lock")?;
    fs2::FileExt::lock_exclusive(&file).context("Impossibile accedere alla coda")?;
    Ok(file)
}
// Assigned under dispatch.lock, so submissions in the same millisecond keep FIFO order.
fn next_order(root: &Path) -> Result<u64> {
    let path = root.join("order.counter");
    let previous = if path.is_file() {
        let bytes: [u8; 8] = fs::read(&path)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Contatore della coda non valido"))?;
        u64::from_le_bytes(bytes)
    } else {
        records(root)?
            .0
            .iter()
            .map(|job| job.submitted_at)
            .max()
            .unwrap_or(0)
    };
    let order = previous.saturating_add(1).max(now());
    let mut temporary = tempfile::NamedTempFile::new_in(root)?;
    temporary.write_all(&order.to_le_bytes())?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(order)
}
fn lock_contended(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::WouldBlock
        || error
            .raw_os_error()
            .zip(fs2::lock_contended_error().raw_os_error())
            .is_some_and(|(actual, expected)| actual == expected)
}
fn store(root: &Path, record: &StoredJob) -> Result<()> {
    retry_store(|| store_once(root, record))
}

fn retry_store(mut operation: impl FnMut() -> Result<()>) -> Result<()> {
    for delay in [25, 50, 100] {
        match operation() {
            Ok(()) => return Ok(()),
            Err(error)
                if cfg!(windows)
                    && error.chain().any(|cause| {
                        cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|error| matches!(error.raw_os_error(), Some(5 | 32 | 33)))
                    }) =>
            {
                thread::sleep(Duration::from_millis(delay))
            }
            Err(error) => return Err(error),
        }
    }
    operation()
}

fn store_once(root: &Path, record: &StoredJob) -> Result<()> {
    let dir = job_dir(root, &record.view.id)?;
    fs::create_dir_all(&dir)?;
    let mut temporary = tempfile::NamedTempFile::new_in(&dir)?;
    serde_json::to_writer_pretty(&mut temporary, record)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(dir.join("job.json"))
        .map_err(|error| error.error)
        .context("Impossibile salvare lo stato della conversione")?;
    Ok(())
}
fn load(root: &Path, id: &str) -> Result<StoredJob> {
    let path = job_dir(root, id)?.join("job.json");
    let record: StoredJob =
        serde_json::from_slice(&fs::read(&path).context("Conversione non trovata nella coda")?)
            .context("Stato della conversione non leggibile")?;
    if record.view.id != id {
        bail!("Identificatore della conversione incoerente")
    }
    Ok(record)
}
fn quarantine_record(path: &Path) -> Result<PathBuf> {
    let mut source = tempfile::TempPath::try_from_path(path)?;
    // This is an existing user record, not a disposable temporary: a failed
    // rename must never let TempPath's destructor delete the original file.
    source.disable_cleanup(true);
    for number in 0..10_000 {
        let target = path.with_file_name(if number == 0 {
            "job.json.unreadable".to_owned()
        } else {
            format!("job.json.unreadable.{number}")
        });
        match source.persist_noclobber(&target) {
            Ok(()) => return Ok(target),
            Err(error) => {
                source = error.path;
                if error.error.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(error.error.into());
                }
            }
        }
    }
    bail!("Troppi record illeggibili conservati per lo stesso lavoro")
}

fn has_quarantined_record(directory: &Path) -> std::io::Result<bool> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name == "job.json.unreadable" || name.starts_with("job.json.unreadable."))
            && entry.file_type()?.is_file()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn records(root: &Path) -> Result<(Vec<StoredJob>, Vec<String>)> {
    let mut jobs = Vec::new();
    let mut warnings = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(format!(
                    "Una voce della coda non è leggibile ed è stata ignorata: {error}"
                ));
                continue;
            }
        };
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !valid_id(&id) {
            continue;
        }
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {}
            Ok(_) => continue,
            Err(error) => {
                warnings.push(format!("{id}: cartella del lavoro non leggibile: {error}"));
                continue;
            }
        }
        let path = entry.path().join("job.json");
        let mut warning = None;
        if path.is_file() {
            match load(root, &id) {
                Ok(job) => jobs.push(job),
                Err(error)
                    if error.chain().any(|cause| {
                        cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound)
                    }) => {}
                Err(error) => {
                    warning = Some(match quarantine_record(&path) {
                        Ok(target) => format!(
                            "{id}: record ignorato e conservato in {}: {error:#}",
                            target.display()
                        ),
                        Err(quarantine_error) => format!(
                            "{id}: record ignorato, impossibile rinominarlo come illeggibile: {error:#}; {quarantine_error:#}"
                        ),
                    });
                }
            }
        }
        if warning.is_none() {
            match has_quarantined_record(&entry.path()) {
                Ok(true) => {
                    warning = Some(format!(
                        "{id}: record illeggibile ignorato e conservato come job.json.unreadable"
                    ))
                }
                Ok(false) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    warning = Some(format!(
                        "{id}: impossibile controllare i record della coda: {error}"
                    ))
                }
            }
        }
        if let Some(warning) = warning {
            warnings.push(warning);
        }
    }
    jobs.sort_by(|left, right| {
        (left.submitted_at, &left.view.id).cmp(&(right.submitted_at, &right.view.id))
    });
    warnings.sort();
    Ok((jobs, warnings))
}

fn cleanup_terminal_jobs(root: &Path, at: u64) -> Result<usize> {
    let root = fs::canonicalize(root)?;
    let mut terminal: Vec<_> = records(&root)?
        .0
        .into_iter()
        .filter(|job| job.view.state.is_terminal())
        .collect();
    terminal.sort_by(|left, right| {
        (right.updated_at, right.submitted_at, &right.view.id).cmp(&(
            left.updated_at,
            left.submitted_at,
            &left.view.id,
        ))
    });
    let mut removed = 0;
    for (index, job) in terminal.iter().enumerate() {
        let retention = match job.view.state {
            JobState::Completed => COMPLETED_RETENTION_MS,
            JobState::Cancelled => CANCELLED_RETENTION_MS,
            JobState::Failed => FAILED_RETENTION_MS,
            _ => continue,
        };
        let expired = job.updated_at != 0 && at.saturating_sub(job.updated_at) > retention;
        if !expired && index < MAX_TERMINAL_JOBS {
            continue;
        }
        match remove_terminal_job(&root, job) {
            Ok(true) => removed += 1,
            Ok(false) => {}
            Err(error) => eprintln!("Impossibile pulire il lavoro {}: {error:#}", job.view.id),
        }
    }
    Ok(removed)
}

fn reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn inside_job_directory(path: &Path, directory: &Path) -> Result<bool> {
    if !path.is_absolute() {
        bail!("Percorso salvato nel lavoro non assoluto");
    }
    if path.starts_with(directory) {
        return Ok(true);
    }
    // A missing output still has an existing parent; canonicalizing ancestors
    // handles Windows case differences and aliases without requiring the file.
    Ok(path.ancestors().any(|ancestor| {
        fs::canonicalize(ancestor).is_ok_and(|resolved| resolved.starts_with(directory))
    }))
}

fn remove_terminal_job(root: &Path, candidate: &StoredJob) -> Result<bool> {
    let directory = job_dir(root, &candidate.view.id)?;
    let metadata = fs::symlink_metadata(&directory)?;
    if !metadata.is_dir()
        || reparse_or_symlink(&metadata)
        || fs::canonicalize(&directory)?.parent() != Some(root)
    {
        bail!("La cartella del lavoro non è una sottocartella locale della coda");
    }
    let record_path = directory.join("job.json");
    let record_metadata = fs::symlink_metadata(&record_path)?;
    if !record_metadata.is_file() || reparse_or_symlink(&record_metadata) {
        bail!("Il record del lavoro non è un file locale della coda");
    }
    let job = load(root, &candidate.view.id)?;
    if !job.view.state.is_terminal() || job.updated_at != candidate.updated_at {
        return Ok(false);
    }
    for path in std::iter::once(&job.view.input)
        .chain(std::iter::once(&job.plan.input))
        .chain(job.view.outputs.iter())
        .chain(job.plan.outputs.iter().map(|output| &output.path))
    {
        if inside_job_directory(path, &directory)? {
            bail!("La cartella contiene un originale o una destinazione: conservata");
        }
    }
    if job.view.state == JobState::Failed {
        for preview in &job.previews {
            preview
                .remove()
                .context("Impossibile rimuovere il segnaposto del lavoro fallito")?;
        }
    }
    let cancel = directory.join("cancel");
    match fs::symlink_metadata(&cancel) {
        Ok(metadata) if metadata.is_file() && !reparse_or_symlink(&metadata) => {
            fs::remove_file(cancel)?
        }
        Ok(_) => bail!("Il segnale di annullamento è stato sostituito: lavoro conservato"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    fs::remove_file(record_path)?;
    // Never recurse: quarantine records, unexpected files and foreign content
    // survive even when the terminal job itself has expired.
    match fs::remove_dir(directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
        Err(error) => return Err(error.into()),
    }
    Ok(true)
}

/// Snapshot without worker/dispatch locks; unreadable records are preserved separately.
pub fn list_jobs() -> Result<Vec<QueueJob>> {
    Ok(list_jobs_with_warnings()?.0)
}

pub fn list_jobs_with_warnings() -> Result<(Vec<QueueJob>, Vec<String>)> {
    jobs_with_warnings_at(&queue_root()?)
}

fn jobs_with_warnings_at(root: &Path) -> Result<(Vec<QueueJob>, Vec<String>)> {
    let (mut items, warnings) = records(root)?;
    items.reverse();
    Ok((items.into_iter().map(|job| job.view).collect(), warnings))
}
pub fn get_job(id: &str) -> Result<QueueJob> {
    Ok(load(&queue_root()?, id)?.view)
}

/// Remove owned Explorer previews without deleting the failed job or its outputs.
pub fn remove_failed_placeholders(id: &str) -> Result<()> {
    remove_failed_placeholders_at(&queue_root()?, id)
}

fn remove_failed_placeholders_at(root: &Path, id: &str) -> Result<()> {
    let _dispatch = dispatch_lock(root)?;
    let job = load(root, id)?;
    if job.view.state != JobState::Failed {
        bail!("Puoi rimuovere i segnaposto soltanto per i lavori falliti");
    }
    if job.previews.iter().any(|preview| {
        !job.plan
            .outputs
            .iter()
            .any(|output| preview.matches_job_output(id, &output.path))
    }) {
        bail!("I segnaposto non corrispondono ai risultati prenotati del lavoro");
    }
    let mut errors = Vec::new();
    for preview in &job.previews {
        if let Err(error) = preview.remove() {
            errors.push(format!("{}: {error:#}", preview.path().display()));
        }
    }
    if !errors.is_empty() {
        bail!(
            "Alcuni segnaposto non sono stati rimossi:\n{}",
            errors.join("\n")
        );
    }
    Ok(())
}

pub fn enqueue(
    files: &[PathBuf],
    settings: &ConversionSettings,
    tools: &ToolPaths,
    executable: &Path,
) -> Result<Submission> {
    submit_at(&queue_root()?, files, settings, tools, executable, true)
}

fn submit_at(
    root: &Path,
    files: &[PathBuf],
    settings: &ConversionSettings,
    tools: &ToolPaths,
    executable: &Path,
    launch_worker: bool,
) -> Result<Submission> {
    if files.is_empty() {
        bail!("Nessun file da convertire")
    }
    fs::create_dir_all(root)?;
    let batch_id = format!("{}-{}", now(), std::process::id());
    let mut submission = Submission {
        jobs: Vec::new(),
        errors: Vec::new(),
    };
    for input in files {
        let result = (|| -> Result<QueueJob> {
            let directory = tempfile::Builder::new().prefix("job-").tempdir_in(root)?;
            let id = directory
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let plan = engine::prepare(input, settings, tools)?;
            let mut previews = Vec::new();
            for output in &plan.outputs {
                match PendingLink::create(
                    &output.path,
                    executable,
                    &id,
                    "In coda — in attesa di conversione",
                ) {
                    Ok(preview) => previews.push(preview),
                    Err(error) => {
                        for preview in &previews {
                            let _ = preview.remove();
                        }
                        let _ = engine::release_prepared(&plan);
                        return Err(error);
                    }
                }
            }
            let mut record = StoredJob {
                view: QueueJob {
                    id,
                    input: plan.input.clone(),
                    format: settings.format.label().into(),
                    outputs: plan
                        .outputs
                        .iter()
                        .map(|output| output.path.clone())
                        .collect(),
                    state: JobState::Queued,
                    progress: 0.0,
                    message: "In coda".into(),
                    error: None,
                },
                plan,
                tools: tools.clone(),
                previews,
                batch_id: batch_id.clone(),
                submitted_at: now(),
                updated_at: now(),
            };
            let persisted = (|| -> Result<()> {
                let _dispatch = dispatch_lock(root)?;
                record.submitted_at = next_order(root)?;
                store(root, &record)
            })();
            if let Err(error) = persisted {
                for preview in &record.previews {
                    let _ = preview.remove();
                }
                let _ = engine::release_prepared(&record.plan);
                return Err(error);
            }
            let _ = directory.keep();
            Ok(record.view)
        })();
        match result {
            Ok(job) => submission.jobs.push(job),
            Err(error) => submission
                .errors
                .push(format!("{}: {error:#}", input.display())),
        }
    }
    if launch_worker && !submission.jobs.is_empty() {
        let _dispatch = dispatch_lock(root)?;
        start_worker_locked(root, executable).context("I lavori sono salvati in coda, ma non è stato possibile avviare il convertitore. Apri Convert Here per riprendere la coda")?;
    }
    Ok(submission)
}

/// Called on app/status startup as well as enqueue, so queued jobs survive an app restart.
pub fn ensure_worker() -> Result<()> {
    let root = queue_root()?;
    let _dispatch = dispatch_lock(&root)?;
    if records(&root)?
        .0
        .iter()
        .any(|job| !job.view.state.is_terminal())
    {
        start_worker_locked(&root, &std::env::current_exe()?)?;
    }
    Ok(())
}
fn start_worker_locked(root: &Path, executable: &Path) -> Result<()> {
    let probe = open_lock(root, "worker.lock")?;
    match fs2::FileExt::try_lock_exclusive(&probe) {
        Ok(()) => {
            drop(probe);
        }
        Err(error) if lock_contended(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    let mut command = Command::new(executable);
    command.arg("--queue-worker");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Avvio del processo della coda")?;
    Ok(())
}

pub fn cancel_job(id: &str) -> Result<()> {
    cancel_at(&queue_root()?, id)
}
fn cancel_at(root: &Path, id: &str) -> Result<()> {
    let _dispatch = dispatch_lock(root)?;
    let mut job = load(root, id)?;
    if job.view.state.is_terminal() {
        return Ok(());
    }
    let mut cancel = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(job_dir(root, id)?.join("cancel"))?;
    cancel.write_all(b"cancel")?;
    if job.view.state == JobState::Queued {
        job.view.state = JobState::Cancelled;
        job.view.message = "Annullato".into();
        job.updated_at = now();
        clean_previews(&job);
        engine::release_prepared(&job.plan)?;
        store(root, &job)?;
    }
    Ok(())
}

pub fn run_worker() -> Result<()> {
    run_worker_at(&queue_root()?, last_report_path().as_deref())
}

fn run_worker_at(root: &Path, report: Option<&Path>) -> Result<()> {
    run_worker_with_store(root, report, store)
}

fn run_worker_with_store(
    root: &Path,
    report: Option<&Path>,
    mut persist: impl FnMut(&Path, &StoredJob) -> Result<()>,
) -> Result<()> {
    let worker = open_lock(root, "worker.lock")?;
    match fs2::FileExt::try_lock_exclusive(&worker) {
        Ok(()) => {}
        Err(error) if lock_contended(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    // The OS releases worker.lock after a crash. A previously Running job is not silently
    // treated as complete, and no external file at its reserved path is ever removed.
    {
        let _dispatch = dispatch_lock(root)?;
        for mut job in records(root)?.0 {
            if job.view.state == JobState::Running {
                job.view.state = JobState::Failed;
                job.view.message = "Conversione interrotta".into();
                job.view.error = Some("Il processo precedente si è interrotto senza salvare lo stato finale. Verifica gli eventuali file di risultato prima di avviare una nuova conversione dall'originale.".into());
                job.updated_at = now();
                for preview in &job.previews {
                    let _ = preview.fail("Conversione interrotta — apri per i dettagli");
                }
                let _ = engine::release_prepared(&job.plan);
                if let Err(error) = persist(root, &job) {
                    eprintln!(
                        "Impossibile salvare il recupero del lavoro {}: {error:#}",
                        job.view.id
                    );
                }
                if let Some(path) = report {
                    let _ = update_report(root, &job.batch_id, path);
                }
            }
        }
        if let Err(error) = cleanup_terminal_jobs(root, now()) {
            eprintln!("Impossibile pulire la cronologia della coda: {error:#}");
        }
    }
    let mut skipped = BTreeSet::new();
    loop {
        let dispatch = dispatch_lock(root)?;
        let next = records(root)?
            .0
            .into_iter()
            .find(|job| job.view.state == JobState::Queued && !skipped.contains(&job.view.id));
        let Some(mut job) = next else {
            if let Err(error) = cleanup_terminal_jobs(root, now()) {
                eprintln!("Impossibile pulire la cronologia della coda: {error:#}");
            }
            // Release worker ownership while enqueue is excluded. This closes the empty-queue
            // handoff race: a submitter will either be seen here or start the next worker.
            drop(worker);
            drop(dispatch);
            return Ok(());
        };
        let cancellation = job_dir(root, &job.view.id)?.join("cancel");
        if cancellation.exists() {
            job.view.state = JobState::Cancelled;
            job.view.message = "Annullato".into();
            clean_previews(&job);
            let _ = engine::release_prepared(&job.plan);
            if let Err(error) = persist(root, &job) {
                eprintln!(
                    "Impossibile salvare l'annullamento del lavoro {}: {error:#}",
                    job.view.id
                );
                skipped.insert(job.view.id.clone());
            }
            drop(dispatch);
            continue;
        }
        job.view.state = JobState::Running;
        job.view.message = "Conversione in corso…".into();
        job.updated_at = now();
        if let Err(error) = persist(root, &job) {
            // Without a durable Running state, a restart would see Queued and
            // could invoke the converter again after an already published output.
            eprintln!(
                "Impossibile avviare il lavoro {} senza salvarne lo stato: {error:#}",
                job.view.id
            );
            skipped.insert(job.view.id.clone());
            drop(dispatch);
            continue;
        }
        drop(dispatch);
        for preview in &job.previews {
            let _ = preview.update("Conversione in corso…");
        }
        let cancel = AtomicBool::new(false);
        let finished = AtomicBool::new(false);
        let mut last_snapshot = Instant::now();
        let mut last_preview = Instant::now();
        let mut last_preview_text = "Conversione in corso…".to_owned();
        let plan = job.plan.clone();
        let tools = job.tools.clone();
        let conversion = thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Relaxed) {
                    if cancellation.exists() {
                        cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                    thread::sleep(Duration::from_millis(75));
                }
            });
            let result = engine::convert_prepared(&plan, &tools, &cancel, |progress| {
                job.view.progress = if progress.fraction.is_finite() {
                    progress.fraction.clamp(0.0, 0.99)
                } else {
                    0.0
                };
                job.view.message = progress.message;
                job.updated_at = now();
                if last_snapshot.elapsed() >= Duration::from_millis(300) {
                    let _ = persist(root, &job);
                    last_snapshot = Instant::now();
                }
                let _ = update_progress_preview(
                    &job,
                    &mut last_preview,
                    &mut last_preview_text,
                    Instant::now(),
                );
            });
            finished.store(true, Ordering::Relaxed);
            result
        });
        match conversion {
            Ok(outputs) => {
                job.view.outputs = outputs;
                job.view.state = JobState::Completed;
                job.view.progress = 1.0;
                job.view.message = "Completato".into();
                clean_previews(&job);
                for output in &job.view.outputs {
                    crate::pending::notify_ready(output);
                }
            }
            Err(error) if cancel.load(Ordering::Relaxed) => {
                job.view.state = JobState::Cancelled;
                job.view.message = "Annullato".into();
                clean_previews(&job);
                let _ = error;
            }
            Err(error) => {
                job.view.state = JobState::Failed;
                job.view.message = "Conversione non riuscita".into();
                job.view.error = Some(format!("{error:#}"));
                for preview in &job.previews {
                    let _ = preview.fail(&format!("Errore — {error:#}"));
                }
            }
        }
        if let Err(error) = engine::release_prepared(&job.plan) {
            job.view
                .message
                .push_str(&format!(" · Pulizia temporanei: {error:#}"));
        }
        job.updated_at = now();
        if let Err(error) = persist(root, &job) {
            // Keep the durable Running record: recovery marks it interrupted
            // rather than retrying a conversion whose output may already exist.
            eprintln!(
                "Impossibile salvare l'esito del lavoro {}: {error:#}",
                job.view.id
            );
            skipped.insert(job.view.id.clone());
        }
        if let Some(path) = report {
            let _ = update_report(root, &job.batch_id, path);
        }
    }
}

fn update_progress_preview(
    job: &StoredJob,
    last_update: &mut Instant,
    last_text: &mut String,
    at: Instant,
) -> Result<bool> {
    if at.saturating_duration_since(*last_update) < PROGRESS_PREVIEW_INTERVAL {
        return Ok(false);
    }
    let Some(preview) = job.previews.first() else {
        return Ok(false);
    };
    let status = if job.view.progress > 0.0 {
        format!("Conversione in corso — {:.0}%", job.view.progress * 100.0)
    } else {
        job.view.message.clone()
    };
    if status == *last_text {
        return Ok(false);
    }
    // Failed COM/ownership checks must also be throttled; otherwise every
    // progress callback retries the same locked or replaced shortcut.
    *last_update = at;
    preview.update(&status)?;
    *last_text = status;
    Ok(true)
}

fn clean_previews(job: &StoredJob) {
    for preview in &job.previews {
        let _ = preview.remove();
    }
}
fn update_report(root: &Path, batch_id: &str, report: &Path) -> Result<()> {
    let jobs: Vec<_> = records(root)?
        .0
        .into_iter()
        .filter(|job| job.batch_id == batch_id)
        .collect();
    let successes: Vec<_> = jobs
        .iter()
        .filter(|job| job.view.state == JobState::Completed)
        .flat_map(|job| job.view.outputs.clone())
        .collect();
    let errors: Vec<_> = jobs
        .iter()
        .filter_map(|job| {
            job.view
                .error
                .as_ref()
                .map(|error| format!("{}: {error}", job.view.input.display()))
        })
        .collect();
    let timestamp =
        time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?;
    let value = serde_json::json!({"timestamp":timestamp,"format":jobs.first().map(|job|job.view.format.as_str()).unwrap_or(""),"successes":successes,"errors":errors});
    let parent = report
        .parent()
        .context("Percorso del resoconto non valido")?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut temporary, &value)?;
    temporary.persist(report).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::OutputFormat;

    #[test]
    fn busy_worker_accepts_new_dispatch_without_starting_another_encoder() {
        let temp = tempfile::tempdir().unwrap();
        let worker = open_lock(temp.path(), "worker.lock").unwrap();
        fs2::FileExt::lock_exclusive(&worker).unwrap();
        let competitor = open_lock(temp.path(), "worker.lock").unwrap();
        assert!(lock_contended(
            &fs2::FileExt::try_lock_exclusive(&competitor).unwrap_err()
        ));
        start_worker_locked(temp.path(), Path::new("nonexistent.exe")).unwrap();
        run_worker_at(temp.path(), None).unwrap();
    }

    #[test]
    fn queue_previews_exist_before_worker_and_publish_exact_names() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("queue");
        let source = temp.path().join("immagine è.png");
        image::RgbaImage::from_pixel(40, 20, image::Rgba([40, 90, 130, 255]))
            .save(&source)
            .unwrap();
        let settings = ConversionSettings {
            format: OutputFormat::Jpg,
            ..Default::default()
        };
        let executable = std::env::current_exe().unwrap();
        let first = submit_at(
            &root,
            std::slice::from_ref(&source),
            &settings,
            &ToolPaths::default(),
            &executable,
            false,
        )
        .unwrap();
        let second = submit_at(
            &root,
            &[source],
            &settings,
            &ToolPaths::default(),
            &executable,
            false,
        )
        .unwrap();
        assert!(first.errors.is_empty() && second.errors.is_empty());
        let first_job = &first.jobs[0];
        let second_job = &second.jobs[0];
        assert_ne!(first_job.outputs, second_job.outputs);
        for job in [first_job, second_job] {
            assert!(!job.outputs[0].exists());
            assert!(PendingLink::path_for(&job.outputs[0]).is_file());
            assert_eq!(load(&root, &job.id).unwrap().view.state, JobState::Queued);
        }
        run_worker_at(&root, Some(&root.join("report-last.json"))).unwrap();
        for job in [first_job, second_job] {
            assert_eq!(
                load(&root, &job.id).unwrap().view.state,
                JobState::Completed
            );
            assert!(image::open(&job.outputs[0]).is_ok());
            assert!(!PendingLink::path_for(&job.outputs[0]).exists());
        }
    }

    #[test]
    fn cancel_queued_and_failed_outputs_have_clear_states() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("queue");
        let source = temp.path().join("broken.png");
        fs::write(&source, b"invalid image").unwrap();
        let settings = ConversionSettings {
            format: OutputFormat::Jpg,
            ..Default::default()
        };
        let exe = std::env::current_exe().unwrap();
        let cancelled = submit_at(
            &root,
            std::slice::from_ref(&source),
            &settings,
            &ToolPaths::default(),
            &exe,
            false,
        )
        .unwrap()
        .jobs
        .remove(0);
        cancel_at(&root, &cancelled.id).unwrap();
        assert_eq!(
            load(&root, &cancelled.id).unwrap().view.state,
            JobState::Cancelled
        );
        assert!(!PendingLink::path_for(&cancelled.outputs[0]).exists());
        let failed = submit_at(
            &root,
            &[source],
            &settings,
            &ToolPaths::default(),
            &exe,
            false,
        )
        .unwrap()
        .jobs
        .remove(0);
        run_worker_at(&root, None).unwrap();
        let state = load(&root, &failed.id).unwrap();
        assert_eq!(state.view.state, JobState::Failed);
        assert!(state.view.error.is_some());
        assert!(PendingLink::path_for(&failed.outputs[0]).exists());
        assert!(!failed.outputs[0].exists());
    }

    #[test]
    fn interrupted_running_job_is_reported_and_preserves_external_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("queue");
        let source = temp.path().join("source.png");
        image::RgbaImage::new(2, 2).save(&source).unwrap();
        let settings = ConversionSettings {
            format: OutputFormat::Jpg,
            ..Default::default()
        };
        let submitted = submit_at(
            &root,
            &[source],
            &settings,
            &ToolPaths::default(),
            &std::env::current_exe().unwrap(),
            false,
        )
        .unwrap();
        let mut record = load(&root, &submitted.jobs[0].id).unwrap();
        record.view.state = JobState::Running;
        store(&root, &record).unwrap();
        fs::write(&record.view.outputs[0], b"external file").unwrap();
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &record.view.id).unwrap().view.state,
            JobState::Failed
        );
        assert_eq!(fs::read(&record.view.outputs[0]).unwrap(), b"external file");
        assert!(!record.plan.outputs[0].marker.exists());
        assert!(get_job("../../outside").is_err());
    }

    fn queued_image(root: &Path, source: &Path) -> QueueJob {
        image::RgbaImage::from_pixel(16, 8, image::Rgba([30, 80, 150, 255]))
            .save(source)
            .unwrap();
        let settings = ConversionSettings {
            format: OutputFormat::Jpg,
            ..Default::default()
        };
        submit_at(
            root,
            &[source.to_owned()],
            &settings,
            &ToolPaths::default(),
            &std::env::current_exe().unwrap(),
            false,
        )
        .unwrap()
        .jobs
        .remove(0)
    }

    #[test]
    fn unreadable_record_is_preserved_and_does_not_block_snapshots_or_valid_jobs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let valid = queued_image(&root, &directory.path().join("valid.png"));
        let invalid = root.join("job-truncated");
        fs::create_dir(&invalid).unwrap();
        let bytes = br#"{"id":"job-truncated"}"#;
        fs::write(invalid.join("job.json"), bytes).unwrap();
        let (jobs, warnings) = jobs_with_warnings_at(&root).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, valid.id);
        assert_eq!(warnings.len(), 1);
        assert!(!invalid.join("job.json").exists());
        assert_eq!(
            fs::read(invalid.join("job.json.unreadable")).unwrap(),
            bytes
        );
        // Startup can quarantine before the GUI's first refresh.
        let (jobs, warnings) = jobs_with_warnings_at(&root).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(warnings.len(), 1);
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &valid.id).unwrap().view.state,
            JobState::Completed
        );
        assert!(image::open(&valid.outputs[0]).is_ok());
        assert_eq!(jobs_with_warnings_at(&root).unwrap().1.len(), 1);
    }

    #[test]
    fn quarantining_never_replaces_earlier_unreadable_records() {
        let directory = tempfile::tempdir().unwrap();
        let invalid = directory.path().join("job-invalid");
        fs::create_dir(&invalid).unwrap();
        fs::write(
            invalid.join("job.json.unreadable"),
            b"earlier corrupt record",
        )
        .unwrap();
        fs::write(invalid.join("job.json"), b"new corrupt record").unwrap();
        let (jobs, warnings) = records(directory.path()).unwrap();
        assert!(jobs.is_empty());
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            fs::read(invalid.join("job.json.unreadable")).unwrap(),
            b"earlier corrupt record"
        );
        assert_eq!(
            fs::read(invalid.join("job.json.unreadable.1")).unwrap(),
            b"new corrupt record"
        );
        assert_eq!(records(directory.path()).unwrap().1.len(), 1);
    }

    #[test]
    fn optional_job_fields_default_without_defaulting_identity_plan_or_state() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let job = queued_image(&root, &directory.path().join("source.png"));
        let record = load(&root, &job.id).unwrap();
        let mut value = serde_json::to_value(&record).unwrap();
        for field in [
            "tools",
            "previews",
            "batch_id",
            "submitted_at",
            "updated_at",
            "outputs",
            "progress",
            "message",
            "error",
        ] {
            value.as_object_mut().unwrap().remove(field);
        }
        let old: StoredJob = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(old.view.id, job.id);
        assert_eq!(old.view.state, JobState::Queued);
        assert_eq!(old.view.progress, 0.0);
        assert!(old.previews.is_empty() && old.view.outputs.is_empty());
        assert_eq!(old.submitted_at, 0);
        let serialized = serde_json::to_value(&old).unwrap();
        for field in [
            "tools",
            "previews",
            "batch_id",
            "submitted_at",
            "updated_at",
            "outputs",
            "progress",
            "message",
            "error",
        ] {
            assert!(
                serialized.get(field).is_some(),
                "serialization omitted {field}"
            );
        }
        for required in ["id", "input", "state", "plan"] {
            let mut invalid = value.clone();
            invalid.as_object_mut().unwrap().remove(required);
            assert!(
                serde_json::from_value::<StoredJob>(invalid).is_err(),
                "defaulted {required}"
            );
        }
        engine::release_prepared(&record.plan).unwrap();
    }

    #[test]
    fn failed_running_store_skips_the_job_without_converting_and_processes_the_next() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let first = queued_image(&root, &directory.path().join("first.png"));
        let second = queued_image(&root, &directory.path().join("second.png"));
        let mut attempts = 0;
        run_worker_with_store(&root, None, |root, job| {
            if job.view.id == first.id && job.view.state == JobState::Running {
                attempts += 1;
                bail!("simulated state write failure");
            }
            store(root, job)
        })
        .unwrap();
        assert_eq!(attempts, 1);
        assert_eq!(load(&root, &first.id).unwrap().view.state, JobState::Queued);
        assert!(!first.outputs[0].exists());
        assert!(
            load(&root, &first.id).unwrap().plan.outputs[0]
                .marker
                .exists()
        );
        assert_eq!(
            load(&root, &second.id).unwrap().view.state,
            JobState::Completed
        );
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &first.id).unwrap().view.state,
            JobState::Completed
        );
        assert!(image::open(&first.outputs[0]).is_ok());
    }

    #[test]
    fn failed_final_store_keeps_running_durable_and_recovery_never_republishes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let first = queued_image(&root, &directory.path().join("first.png"));
        let second = queued_image(&root, &directory.path().join("second.png"));
        run_worker_with_store(&root, None, |root, job| {
            if job.view.id == first.id && job.view.state == JobState::Completed {
                bail!("simulated final state failure");
            }
            store(root, job)
        })
        .unwrap();
        assert_eq!(
            load(&root, &first.id).unwrap().view.state,
            JobState::Running
        );
        assert_eq!(
            load(&root, &second.id).unwrap().view.state,
            JobState::Completed
        );
        let output = fs::read(&first.outputs[0]).unwrap();
        run_worker_at(&root, None).unwrap();
        assert_eq!(load(&root, &first.id).unwrap().view.state, JobState::Failed);
        assert_eq!(fs::read(&first.outputs[0]).unwrap(), output);
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "jpg"))
                .count(),
            2
        );
    }

    #[test]
    fn recovery_and_cancellation_store_failures_do_not_block_other_jobs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let running = queued_image(&root, &directory.path().join("running.png"));
        let cancelled = queued_image(&root, &directory.path().join("cancelled.png"));
        let valid = queued_image(&root, &directory.path().join("valid.png"));
        let mut record = load(&root, &running.id).unwrap();
        record.view.state = JobState::Running;
        store(&root, &record).unwrap();
        fs::write(
            job_dir(&root, &cancelled.id).unwrap().join("cancel"),
            b"cancel",
        )
        .unwrap();
        run_worker_with_store(&root, None, |root, job| {
            if job.view.id == running.id || job.view.id == cancelled.id {
                bail!("simulated recovery/cancellation write failure");
            }
            store(root, job)
        })
        .unwrap();
        assert_eq!(
            load(&root, &valid.id).unwrap().view.state,
            JobState::Completed
        );
        assert!(!running.outputs[0].exists());
        assert!(!cancelled.outputs[0].exists());
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &running.id).unwrap().view.state,
            JobState::Failed
        );
        assert_eq!(
            load(&root, &cancelled.id).unwrap().view.state,
            JobState::Cancelled
        );
    }

    #[test]
    fn unrelated_store_errors_are_not_retried() {
        let mut attempts = 0;
        assert!(
            retry_store(|| {
                attempts += 1;
                Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
                    .context("wrapped failure")
            })
            .is_err()
        );
        assert_eq!(attempts, 1);
    }

    #[cfg(windows)]
    #[test]
    fn windows_sharing_store_retries_are_bounded_and_unwrap_context() {
        for code in [5, 32, 33] {
            let mut attempts = 0;
            let result = retry_store(|| {
                attempts += 1;
                Err(std::io::Error::from_raw_os_error(code)).context("wrapped Windows failure")
            });
            assert!(result.is_err());
            assert_eq!(attempts, 4);
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_transient_job_lock_retries_then_saves_the_record() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let job = queued_image(&root, &directory.path().join("source.png"));
        let mut record = load(&root, &job.id).unwrap();
        record.view.message = "Updated after transient lock".into();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(job_dir(&root, &job.id).unwrap().join("job.json"))
            .unwrap();
        let (release, receiver) = std::sync::mpsc::channel();
        let unlock = thread::spawn(move || {
            receiver.recv().unwrap();
            thread::sleep(Duration::from_millis(35));
            drop(lock);
        });
        let mut attempts = 0;
        retry_store(|| {
            attempts += 1;
            let result = store_once(&root, &record);
            if attempts == 1 {
                assert!(result.is_err());
                release.send(()).unwrap();
            }
            result
        })
        .unwrap();
        unlock.join().unwrap();
        assert!(attempts > 1);
        assert_eq!(
            load(&root, &job.id).unwrap().view.message,
            record.view.message
        );
    }

    #[cfg(windows)]
    #[test]
    fn failed_quarantine_keeps_the_original_and_valid_jobs_keep_running() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let valid = queued_image(&root, &directory.path().join("source.png"));
        let invalid = root.join("job-locked");
        fs::create_dir(&invalid).unwrap();
        fs::write(invalid.join("job.json"), b"corrupt record").unwrap();
        let locked = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(invalid.join("job.json"))
            .unwrap();
        let (jobs, warnings) = jobs_with_warnings_at(&root).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("impossibile rinominarlo"));
        assert_eq!(
            fs::read(invalid.join("job.json")).unwrap(),
            b"corrupt record"
        );
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &valid.id).unwrap().view.state,
            JobState::Completed
        );
        drop(locked);
        assert_eq!(records(&root).unwrap().1.len(), 1);
        assert_eq!(
            fs::read(invalid.join("job.json.unreadable")).unwrap(),
            b"corrupt record"
        );
    }

    fn history_record(
        root: &Path,
        template: &StoredJob,
        id: &str,
        state: JobState,
        updated_at: u64,
    ) -> StoredJob {
        let mut job: StoredJob =
            serde_json::from_value(serde_json::to_value(template).unwrap()).unwrap();
        job.view.id = id.into();
        job.view.state = state;
        job.previews.clear();
        job.updated_at = updated_at;
        job.submitted_at = updated_at;
        store(root, &job).unwrap();
        job
    }

    #[test]
    fn history_cleanup_obeys_state_ages_and_keeps_active_jobs_outputs_and_unknown_dates() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let template = load(&root, &queued.id).unwrap();
        let at = 2_000_000_000_000;
        let day = 24 * 60 * 60 * 1_000;
        for (id, state, age) in [
            ("job-old-completed", JobState::Completed, 8 * day),
            ("job-old-cancelled", JobState::Cancelled, 8 * day),
            ("job-old-failed", JobState::Failed, 31 * day),
            ("job-recent-failed", JobState::Failed, 8 * day),
            ("job-recent-completed", JobState::Completed, 6 * day),
            ("job-old-queued", JobState::Queued, 100 * day),
            ("job-old-running", JobState::Running, 100 * day),
        ] {
            history_record(&root, &template, id, state, at - age);
        }
        history_record(&root, &template, "job-unknown-date", JobState::Completed, 0);
        let original = fs::read(&queued.input).unwrap();
        fs::write(&queued.outputs[0], b"keep converted output").unwrap();
        assert_eq!(cleanup_terminal_jobs(&root, at).unwrap(), 3);
        for id in ["job-old-completed", "job-old-cancelled", "job-old-failed"] {
            assert!(!root.join(id).exists());
        }
        for id in [
            "job-recent-failed",
            "job-recent-completed",
            "job-old-queued",
            "job-old-running",
            "job-unknown-date",
        ] {
            assert!(root.join(id).join("job.json").is_file());
        }
        assert_eq!(fs::read(&queued.input).unwrap(), original);
        assert_eq!(
            fs::read(&queued.outputs[0]).unwrap(),
            b"keep converted output"
        );
        assert!(template.plan.outputs[0].marker.is_file());
        assert!(PendingLink::path_for(&queued.outputs[0]).is_file());
    }

    #[test]
    fn history_cleanup_caps_terminal_records_without_counting_active_jobs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let template = load(&root, &queued.id).unwrap();
        for index in 0..MAX_TERMINAL_JOBS + 5 {
            let mut job = history_record(
                &root,
                &template,
                &format!("job-history-{index:04}"),
                JobState::Completed,
                0,
            );
            job.submitted_at = index as u64;
            store(&root, &job).unwrap();
        }
        assert_eq!(cleanup_terminal_jobs(&root, now()).unwrap(), 5);
        let jobs = records(&root).unwrap().0;
        assert_eq!(
            jobs.iter()
                .filter(|job| job.view.state.is_terminal())
                .count(),
            MAX_TERMINAL_JOBS
        );
        assert_eq!(
            load(&root, &queued.id).unwrap().view.state,
            JobState::Queued
        );
        for index in 0..5 {
            assert!(!root.join(format!("job-history-{index:04}")).exists());
        }
        assert!(root.join("job-history-0005/job.json").is_file());
    }

    #[test]
    fn history_cleanup_preserves_quarantines_foreign_files_and_files_declared_inside_job_dirs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let template = load(&root, &queued.id).unwrap();
        let at = 2_000_000_000_000;
        for id in ["job-quarantine", "job-foreign", "job-output", "job-input"] {
            history_record(&root, &template, id, JobState::Completed, 1);
        }
        fs::write(
            root.join("job-quarantine/job.json.unreadable"),
            b"preserve corrupt evidence",
        )
        .unwrap();
        fs::write(root.join("job-foreign/notes.txt"), b"foreign notes").unwrap();
        let mut output_job = load(&root, "job-output").unwrap();
        output_job.plan.outputs[0].path = fs::canonicalize(root.join("job-output"))
            .unwrap()
            .join("result.jpg");
        fs::write(&output_job.plan.outputs[0].path, b"protected output").unwrap();
        store(&root, &output_job).unwrap();
        let mut input_job = load(&root, "job-input").unwrap();
        input_job.view.input = fs::canonicalize(root.join("job-input"))
            .unwrap()
            .join("source.png");
        fs::write(&input_job.view.input, b"protected original").unwrap();
        store(&root, &input_job).unwrap();
        assert_eq!(cleanup_terminal_jobs(&root, at).unwrap(), 2);
        assert!(!root.join("job-quarantine/job.json").exists());
        assert!(!root.join("job-foreign/job.json").exists());
        assert_eq!(
            fs::read(root.join("job-quarantine/job.json.unreadable")).unwrap(),
            b"preserve corrupt evidence"
        );
        assert_eq!(
            fs::read(root.join("job-foreign/notes.txt")).unwrap(),
            b"foreign notes"
        );
        assert_eq!(
            fs::read(&output_job.plan.outputs[0].path).unwrap(),
            b"protected output"
        );
        assert_eq!(
            fs::read(&input_job.view.input).unwrap(),
            b"protected original"
        );
        assert!(root.join("job-output/job.json").is_file());
        assert!(root.join("job-input/job.json").is_file());
        assert_eq!(records(&root).unwrap().1.len(), 1);
    }

    #[test]
    fn failed_history_removes_only_owned_previews_and_keeps_metadata_on_preview_failure() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let owned = queued_image(&root, &directory.path().join("owned.png"));
        let replaced = queued_image(&root, &directory.path().join("replaced.png"));
        for job in [&owned, &replaced] {
            let mut record = load(&root, &job.id).unwrap();
            record.view.state = JobState::Failed;
            record.updated_at = 1;
            store(&root, &record).unwrap();
        }
        let foreign_link = PendingLink::path_for(&replaced.outputs[0]);
        fs::write(&foreign_link, b"replaced by another program").unwrap();
        assert_eq!(cleanup_terminal_jobs(&root, now()).unwrap(), 1);
        assert!(!root.join(&owned.id).exists());
        assert!(!PendingLink::path_for(&owned.outputs[0]).exists());
        assert!(root.join(&replaced.id).join("job.json").is_file());
        assert_eq!(
            fs::read(foreign_link).unwrap(),
            b"replaced by another program"
        );
    }

    #[cfg(windows)]
    #[test]
    fn failed_placeholder_removal_is_idempotent_and_preserves_job_source_and_output() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let mut job = load(&root, &queued.id).unwrap();
        job.view.state = JobState::Failed;
        store(&root, &job).unwrap();
        fs::write(&queued.outputs[0], b"result created by another program").unwrap();
        let source = fs::read(&queued.input).unwrap();
        let record = fs::read(root.join(&queued.id).join("job.json")).unwrap();
        let reservation = fs::read(&job.plan.outputs[0].marker).unwrap();

        remove_failed_placeholders_at(&root, &queued.id).unwrap();
        remove_failed_placeholders_at(&root, &queued.id).unwrap();

        assert!(!PendingLink::path_for(&queued.outputs[0]).exists());
        assert_eq!(fs::read(&queued.input).unwrap(), source);
        assert_eq!(
            fs::read(&queued.outputs[0]).unwrap(),
            b"result created by another program"
        );
        assert_eq!(
            fs::read(root.join(&queued.id).join("job.json")).unwrap(),
            record
        );
        assert_eq!(fs::read(&job.plan.outputs[0].marker).unwrap(), reservation);
    }

    #[cfg(windows)]
    #[test]
    fn placeholder_removal_rejects_every_state_other_than_failed() {
        for state in [
            JobState::Queued,
            JobState::Running,
            JobState::Completed,
            JobState::Cancelled,
        ] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().join("queue");
            let queued = queued_image(&root, &directory.path().join("source.png"));
            let mut job = load(&root, &queued.id).unwrap();
            job.view.state = state;
            store(&root, &job).unwrap();
            let record = fs::read(root.join(&queued.id).join("job.json")).unwrap();
            let preview = fs::read(job.previews[0].path()).unwrap();

            let error = remove_failed_placeholders_at(&root, &queued.id).unwrap_err();

            assert!(error.to_string().contains("soltanto per i lavori falliti"));
            assert_eq!(fs::read(job.previews[0].path()).unwrap(), preview);
            assert_eq!(
                fs::read(root.join(&queued.id).join("job.json")).unwrap(),
                record
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn failed_placeholder_removal_keeps_replacements_and_cleans_other_owned_links() {
        for valid_foreign_link in [true, false] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().join("queue");
            let queued = queued_image(&root, &directory.path().join("source.png"));
            let mut job = load(&root, &queued.id).unwrap();
            let second_output = directory.path().join("second.jpg");
            let mut second_reserved = job.plan.outputs[0].clone();
            second_reserved.path = second_output.clone();
            job.plan.outputs.push(second_reserved);
            job.view.outputs.push(second_output.clone());
            job.previews.push(
                PendingLink::create(
                    &second_output,
                    &std::env::current_exe().unwrap(),
                    &queued.id,
                    "Errore",
                )
                .unwrap(),
            );
            job.view.state = JobState::Failed;
            store(&root, &job).unwrap();
            let record = fs::read(root.join(&queued.id).join("job.json")).unwrap();
            job.previews[0].remove().unwrap();
            if valid_foreign_link {
                PendingLink::create(
                    &queued.outputs[0],
                    &std::env::current_exe().unwrap(),
                    "job-foreign",
                    "Collegamento di un altro lavoro",
                )
                .unwrap();
            } else {
                fs::write(job.previews[0].path(), b"foreign non-link content").unwrap();
            }
            let replacement = fs::read(job.previews[0].path()).unwrap();
            let source = fs::read(&queued.input).unwrap();

            let error = remove_failed_placeholders_at(&root, &queued.id).unwrap_err();

            assert!(error.to_string().contains("non sono stati rimossi"));
            assert_eq!(fs::read(job.previews[0].path()).unwrap(), replacement);
            assert!(!job.previews[1].path().exists());
            assert_eq!(fs::read(&queued.input).unwrap(), source);
            assert_eq!(
                fs::read(root.join(&queued.id).join("job.json")).unwrap(),
                record
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn failed_placeholder_removal_rejects_inconsistent_job_path_and_output() {
        for field in ["job_id", "path", "output"] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().join("queue");
            let queued = queued_image(&root, &directory.path().join("source.png"));
            let mut job = load(&root, &queued.id).unwrap();
            job.view.state = JobState::Failed;
            let mut value = serde_json::to_value(&job).unwrap();
            value["previews"][0][field] = if field == "job_id" {
                serde_json::json!("job-foreign")
            } else {
                serde_json::json!(&queued.input)
            };
            let changed: StoredJob = serde_json::from_value(value).unwrap();
            store(&root, &changed).unwrap();
            let record = fs::read(root.join(&queued.id).join("job.json")).unwrap();
            let source = fs::read(&queued.input).unwrap();
            let preview = fs::read(job.previews[0].path()).unwrap();

            let error = remove_failed_placeholders_at(&root, &queued.id).unwrap_err();

            assert!(error.to_string().contains("risultati prenotati"));
            assert_eq!(fs::read(&queued.input).unwrap(), source);
            assert_eq!(fs::read(job.previews[0].path()).unwrap(), preview);
            assert_eq!(
                fs::read(root.join(&queued.id).join("job.json")).unwrap(),
                record
            );
        }
    }

    #[test]
    fn worker_cleans_old_history_at_startup_and_again_when_the_queue_becomes_empty() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let template = load(&root, &queued.id).unwrap();
        history_record(&root, &template, "job-old-history", JobState::Completed, 1);
        run_worker_with_store(&root, None, |root, job| {
            assert!(!root.join("job-old-history").exists());
            if job.view.state == JobState::Completed {
                let mut old: StoredJob =
                    serde_json::from_value(serde_json::to_value(job).unwrap()).unwrap();
                old.updated_at = 1;
                return store(root, &old);
            }
            store(root, job)
        })
        .unwrap();
        assert!(!root.join(&queued.id).exists());
        assert!(image::open(&queued.outputs[0]).is_ok());
        assert!(queued.input.is_file());
    }

    #[cfg(windows)]
    #[test]
    fn history_cleanup_rejects_junctions_and_leaves_the_external_directory_intact() {
        use std::os::windows::process::CommandExt;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let queued = queued_image(&root, &directory.path().join("source.png"));
        let template = load(&root, &queued.id).unwrap();
        let outside = directory.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let mut job: StoredJob =
            serde_json::from_value(serde_json::to_value(template).unwrap()).unwrap();
        job.view.id = "job-junction".into();
        job.view.state = JobState::Completed;
        job.updated_at = 1;
        fs::write(outside.join("job.json"), serde_json::to_vec(&job).unwrap()).unwrap();
        fs::write(outside.join("keep.txt"), b"outside queue").unwrap();
        let link = root.join("job-junction");
        let result = Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link)
            .arg(&outside)
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(remove_terminal_job(&fs::canonicalize(&root).unwrap(), &job).is_err());
        cleanup_terminal_jobs(&root, now()).unwrap();
        assert!(outside.join("job.json").is_file());
        assert_eq!(
            fs::read(outside.join("keep.txt")).unwrap(),
            b"outside queue"
        );
        fs::remove_dir(link).unwrap();
    }

    #[cfg(windows)]
    fn progress_job(root: &Path, source: &Path) -> StoredJob {
        let queued = queued_image(root, source);
        let mut job = load(root, &queued.id).unwrap();
        for index in 2..=3 {
            let output = source.with_file_name(format!("progress-{index}.jpg"));
            job.previews.push(
                PendingLink::create(
                    &output,
                    &std::env::current_exe().unwrap(),
                    &queued.id,
                    "Conversione in corso…",
                )
                .unwrap(),
            );
        }
        for preview in &job.previews {
            preview.update("Conversione in corso…").unwrap();
        }
        job
    }

    #[cfg(windows)]
    #[test]
    fn progress_updates_only_the_first_real_preview_after_five_seconds_and_changed_text() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let mut job = progress_job(&root, &directory.path().join("source.png"));
        let identities: Vec<_> = job
            .previews
            .iter()
            .map(|preview| engine::file_identity(&File::open(preview.path()).unwrap()).unwrap())
            .collect();
        let before: Vec<_> = job
            .previews
            .iter()
            .map(|preview| fs::read(preview.path()).unwrap())
            .collect();
        let at = Instant::now();
        let mut last_update = at - Duration::from_secs(6);
        let mut last_text = "Conversione in corso…".to_owned();
        job.view.progress = 0.1;
        assert!(update_progress_preview(&job, &mut last_update, &mut last_text, at).unwrap());
        assert_eq!(last_text, "Conversione in corso — 10%");
        assert_ne!(fs::read(job.previews[0].path()).unwrap(), before[0]);
        let current = engine::file_identity(&File::open(job.previews[0].path()).unwrap()).unwrap();
        assert_ne!(identities[0], current);
        let first_update = fs::read(job.previews[0].path()).unwrap();
        job.view.progress = 0.2;
        assert!(
            !update_progress_preview(
                &job,
                &mut last_update,
                &mut last_text,
                at + Duration::from_secs(4)
            )
            .unwrap()
        );
        job.view.progress = 0.1;
        assert!(
            !update_progress_preview(
                &job,
                &mut last_update,
                &mut last_text,
                at + Duration::from_secs(6)
            )
            .unwrap()
        );
        assert_eq!(last_update, at);
        assert_eq!(fs::read(job.previews[0].path()).unwrap(), first_update);
        assert_eq!(
            engine::file_identity(&File::open(job.previews[0].path()).unwrap()).unwrap(),
            current
        );
        job.view.progress = 0.2;
        assert!(
            update_progress_preview(
                &job,
                &mut last_update,
                &mut last_text,
                at + Duration::from_secs(6)
            )
            .unwrap()
        );
        assert_eq!(last_text, "Conversione in corso — 20%");
        for index in 1..=2 {
            assert_eq!(fs::read(job.previews[index].path()).unwrap(), before[index]);
            assert_eq!(
                engine::file_identity(&File::open(job.previews[index].path()).unwrap()).unwrap(),
                identities[index]
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn failed_progress_preview_attempts_are_also_throttled_without_accepting_the_text() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let mut job = progress_job(&root, &directory.path().join("source.png"));
        fs::write(job.previews[0].path(), b"foreign replacement").unwrap();
        let at = Instant::now();
        let mut last_update = at - Duration::from_secs(6);
        let mut last_text = "Conversione in corso…".to_owned();
        job.view.progress = 0.2;
        assert!(update_progress_preview(&job, &mut last_update, &mut last_text, at).is_err());
        assert_eq!(last_update, at);
        assert_eq!(last_text, "Conversione in corso…");
        assert!(
            !update_progress_preview(
                &job,
                &mut last_update,
                &mut last_text,
                at + Duration::from_secs(1)
            )
            .unwrap()
        );
        assert!(
            update_progress_preview(
                &job,
                &mut last_update,
                &mut last_text,
                at + Duration::from_secs(5)
            )
            .is_err()
        );
        assert_eq!(
            fs::read(job.previews[0].path()).unwrap(),
            b"foreign replacement"
        );
    }

    #[cfg(windows)]
    #[test]
    fn worker_failure_still_changes_every_preview_to_the_red_error_state() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("queue");
        let source = directory.path().join("source.png");
        let job = progress_job(&root, &source);
        store(&root, &job).unwrap();
        fs::write(source, b"invalid image").unwrap();
        run_worker_at(&root, None).unwrap();
        assert_eq!(
            load(&root, &job.view.id).unwrap().view.state,
            JobState::Failed
        );
        let error_text: Vec<_> = "Errore —"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        for preview in &job.previews {
            let bytes = fs::read(preview.path()).unwrap();
            assert!(
                bytes
                    .windows(error_text.len())
                    .any(|window| window == error_text)
            );
            // MS-SHLLINK ShellLinkHeader stores IconIndex at byte 0x38.
            assert_eq!(
                i32::from_le_bytes(bytes[0x38..0x3c].try_into().unwrap()),
                -3
            );
        }
    }
}

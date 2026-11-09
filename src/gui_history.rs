//! Bounded GUI history. Jobs are data only; no selected/output file is opened.
use anyhow::{Context, Result, ensure};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
};
use the_converter::{i18n::tr, queue::QueueJob};

const MAX_BYTES: u64 = 1024 * 1024;
const MAX_JOBS: usize = 300;
const SCHEMA: &str = "convert-here-gui-history";

pub(crate) fn path() -> Option<PathBuf> {
    the_converter::i18n::preferences_path().map(|path| path.with_file_name("history.json"))
}

#[derive(Clone)]
pub(crate) struct AsyncHistory {
    sender: mpsc::Sender<Request>,
    state: Arc<Mutex<AsyncState>>,
    refreshing: Arc<AtomicBool>,
    pending_records: Arc<AtomicUsize>,
}

static REGISTRY: OnceLock<Mutex<BTreeMap<PathBuf, AsyncHistory>>> = OnceLock::new();

enum Request {
    Refresh,
    Record(Box<QueueJob>),
}

struct AsyncState {
    revision: u64,
    jobs: Vec<QueueJob>,
    pending: Vec<QueueJob>,
    write_errors: BTreeMap<String, String>,
    error: Option<String>,
}

pub(crate) struct Snapshot {
    pub revision: u64,
    pub jobs: Vec<QueueJob>,
    pub error: Option<String>,
}

fn merged_jobs(pending: &[QueueJob], stored: Vec<QueueJob>) -> Vec<QueueJob> {
    let mut ids = BTreeSet::new();
    pending
        .iter()
        .cloned()
        .chain(stored)
        .filter(|job| ids.insert(job.id.clone()))
        .take(MAX_JOBS)
        .collect()
}

impl AsyncHistory {
    pub(crate) fn open(path: PathBuf) -> Result<Self> {
        let mut registry = REGISTRY
            .get_or_init(|| Mutex::new(BTreeMap::new()))
            .lock()
            .map_err(|_| {
                anyhow::anyhow!(tr("Cronologia non disponibile", "History is unavailable"))
            })?;
        if let Some(history) = registry.get(&path) {
            return Ok(history.clone());
        }
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(AsyncState {
            revision: 1,
            jobs: vec![],
            pending: vec![],
            write_errors: BTreeMap::new(),
            error: None,
        }));
        let refreshing = Arc::new(AtomicBool::new(false));
        let pending_records = Arc::new(AtomicUsize::new(0));
        let thread_state = state.clone();
        let thread_refreshing = refreshing.clone();
        let thread_pending = pending_records.clone();
        let thread_path = path.clone();
        std::thread::Builder::new()
            .name("gui-history".into())
            .spawn(move || {
                while let Ok(request) = receiver.recv() {
                    // File locks and all disk access stay outside the state mutex.
                    let (result, recorded, refreshed) = match request {
                        Request::Refresh => (load(&thread_path), None, true),
                        Request::Record(job) => {
                            let result = record(&thread_path, &job);
                            (result, Some(job.id), false)
                        }
                    };
                    let was_record = recorded.is_some();
                    if let Ok(mut state) = thread_state.lock() {
                        match result {
                            Ok(jobs) => {
                                if let Some(id) = recorded {
                                    state.pending.retain(|job| job.id != id);
                                    state.write_errors.remove(&id);
                                }
                                state.jobs = merged_jobs(&state.pending, jobs);
                                state.error = state.write_errors.values().next().cloned();
                            }
                            Err(error) => {
                                let error = format!("{error:#}");
                                if let Some(id) = recorded {
                                    state.write_errors.insert(id, error.clone());
                                }
                                state.error = Some(error);
                            }
                        }
                        state.revision = state.revision.wrapping_add(1);
                    }
                    if refreshed {
                        thread_refreshing.store(false, Ordering::Release);
                    }
                    if was_record {
                        thread_pending.fetch_sub(1, Ordering::AcqRel);
                    }
                }
            })?;
        let history = Self {
            sender,
            state,
            refreshing,
            pending_records,
        };
        history.refresh();
        registry.insert(path, history.clone());
        Ok(history)
    }

    pub(crate) fn refresh(&self) {
        if !self.refreshing.swap(true, Ordering::AcqRel)
            && self.sender.send(Request::Refresh).is_err()
        {
            self.refreshing.store(false, Ordering::Release);
            if let Ok(mut state) = self.state.lock() {
                state.error = Some(
                    tr(
                        "Lettura cronologia interrotta",
                        "History reading was interrupted",
                    )
                    .into(),
                );
                state.revision = state.revision.wrapping_add(1);
            }
        }
    }

    pub(crate) fn snapshot_after(&self, revision: u64) -> Option<Snapshot> {
        let state = self.state.lock().ok()?;
        (state.revision != revision).then(|| Snapshot {
            revision: state.revision,
            jobs: state.jobs.clone(),
            error: state.error.clone(),
        })
    }

    fn enqueue(&self, job: QueueJob) -> Result<()> {
        validate_job(&job)?;
        {
            let mut state = self.state.lock().map_err(|_| {
                anyhow::anyhow!(tr("Cronologia non disponibile", "History is unavailable"))
            })?;
            state.pending.retain(|previous| previous.id != job.id);
            state.pending.insert(0, job.clone());
            state.pending.truncate(MAX_JOBS);
            let stored = std::mem::take(&mut state.jobs);
            state.jobs = merged_jobs(&state.pending, stored);
            state.revision = state.revision.wrapping_add(1);
        }
        let id = job.id.clone();
        self.pending_records.fetch_add(1, Ordering::AcqRel);
        if self.sender.send(Request::Record(Box::new(job))).is_err() {
            self.pending_records.fetch_sub(1, Ordering::AcqRel);
            let error = tr(
                "Salvataggio cronologia interrotto",
                "History saving was interrupted",
            )
            .to_string();
            if let Ok(mut state) = self.state.lock() {
                state.write_errors.insert(id, error.clone());
                state.error = Some(error.clone());
                state.revision = state.revision.wrapping_add(1);
            }
            anyhow::bail!(error);
        }
        Ok(())
    }
}

/// Enqueue only. Successful conversions never wait for a history file lock.
pub(crate) fn record_async(path: PathBuf, job: QueueJob) -> Result<()> {
    AsyncHistory::open(path)?.enqueue(job)
}

#[derive(Clone)]
struct CloseFlush {
    started: std::time::Instant,
    timed_out: bool,
    allow_close: bool,
}

/// Keep repainting while records flush. After five seconds a second close
/// explicitly leaves without waiting for a foreign file lock.
pub(crate) fn guard_close(
    ctx: &egui::Context,
    work_busy: bool,
    close_when_idle: bool,
) -> Option<(String, bool)> {
    let pending = path()
        .and_then(|path| {
            REGISTRY
                .get()?
                .lock()
                .ok()?
                .get(&path)
                .map(|history| history.pending_records.load(Ordering::Acquire))
        })
        .unwrap_or(0);
    guard_close_pending(ctx, work_busy, close_when_idle, pending)
}

fn guard_close_pending(
    ctx: &egui::Context,
    work_busy: bool,
    close_when_idle: bool,
    pending: usize,
) -> Option<(String, bool)> {
    let id = egui::Id::new("gui_history_close_flush");
    if work_busy {
        ctx.data_mut(|data| data.remove::<CloseFlush>(id));
        return None;
    }
    let mut state = ctx.data_mut(|data| data.get_temp::<CloseFlush>(id));
    let requested = ctx.input(|input| input.viewport().close_requested());
    if pending == 0 {
        if let Some(state) = state {
            ctx.data_mut(|data| data.remove::<CloseFlush>(id));
            if state.timed_out {
                return Some((
                    tr(
                        "Cronologia salvata. Puoi chiudere la finestra.",
                        "History saved. You can close the window.",
                    )
                    .into(),
                    false,
                ));
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        return None;
    }
    if state.is_none() && !requested && !close_when_idle {
        return None;
    }
    let state = state.get_or_insert_with(|| CloseFlush {
        started: std::time::Instant::now(),
        timed_out: false,
        allow_close: false,
    });
    if state.allow_close {
        return None;
    }
    if state.timed_out {
        if requested {
            state.allow_close = true;
            ctx.data_mut(|data| data.insert_temp(id, state.clone()));
            return None;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    } else if state.started.elapsed() >= std::time::Duration::from_secs(5) {
        state.timed_out = true;
        // Give the warning a visible frame before a new close can exit.
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.request_repaint();
    } else {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
    ctx.data_mut(|data| data.insert_temp(id, state.clone()));
    Some(if state.timed_out {
        (tr("La cronologia è ancora in attesa e potrebbe non essere salvata. Chiudi di nuovo per uscire comunque.", "History is still pending and may not be saved. Close again to exit anyway.").into(), true)
    } else {
        (
            tr("Salvataggio della cronologia…", "Saving history…").into(),
            false,
        )
    })
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    schema: String,
    version: u32,
    jobs: Vec<QueueJob>,
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

fn open_lock(path: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent(path).join("history.lock"))?)
}

fn validate_job(job: &QueueJob) -> Result<()> {
    ensure!(
        job.state.is_terminal(),
        tr(
            "La cronologia accetta solo conversioni terminate",
            "History accepts only finished conversions"
        )
    );
    ensure!(
        !job.id.is_empty() && job.progress.is_finite(),
        tr("Voce della cronologia non valida", "Invalid history entry")
    );
    Ok(())
}

fn load_unlocked(path: &Path) -> Result<Vec<QueueJob>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        tr(
            "Il file della cronologia supera 1 MiB",
            "The history file exceeds 1 MiB"
        )
    );
    let history: History = serde_json::from_slice(&bytes).context(tr(
        "Il file della cronologia non è valido; i dati sono stati conservati",
        "The history file is invalid; its data has been preserved",
    ))?;
    ensure!(
        history.schema == SCHEMA && history.version == 1 && history.jobs.len() <= MAX_JOBS,
        tr(
            "Formato o versione della cronologia non supportati",
            "Unsupported history format or version"
        )
    );
    let mut ids = BTreeSet::new();
    for job in &history.jobs {
        validate_job(job)?;
        ensure!(
            ids.insert(&job.id),
            tr(
                "Identificatore duplicato nella cronologia",
                "Duplicate history identifier"
            )
        );
    }
    Ok(history.jobs)
}

pub(crate) fn load(path: &Path) -> Result<Vec<QueueJob>> {
    if !path.try_exists()? {
        return Ok(vec![]);
    }
    let lock = open_lock(path)?;
    fs2::FileExt::lock_shared(&lock)?;
    load_unlocked(path)
}

fn encoded(jobs: &[QueueJob]) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    struct Wire<'a> {
        schema: &'static str,
        version: u32,
        jobs: &'a [QueueJob],
    }
    Ok(serde_json::to_vec(&Wire {
        schema: SCHEMA,
        version: 1,
        jobs,
    })?)
}

pub(crate) fn record(path: &Path, job: &QueueJob) -> Result<Vec<QueueJob>> {
    validate_job(job)?;
    ensure!(
        path.file_name().is_some(),
        tr(
            "Percorso della cronologia non valido",
            "Invalid history path"
        )
    );
    std::fs::create_dir_all(parent(path))?;
    let lock = open_lock(path)?;
    fs2::FileExt::lock_exclusive(&lock)?;
    // A corrupt/unsupported existing file is never replaced with fresh data.
    let mut jobs = load_unlocked(path)?;
    let mut incoming = job.clone();
    if encoded(std::slice::from_ref(&incoming))?.len() as u64 > MAX_BYTES {
        let count = incoming.outputs.len();
        incoming.outputs.truncate(32);
        if encoded(std::slice::from_ref(&incoming))?.len() as u64 > MAX_BYTES {
            incoming.outputs.truncate(1);
        }
        if incoming.outputs.len() < count {
            incoming.message.push_str(&format!(
                "\n{}: {} / {count}",
                tr(
                    "Percorsi dei risultati conservati in cronologia",
                    "Output paths retained in history"
                ),
                incoming.outputs.len()
            ));
        }
        ensure!(
            encoded(std::slice::from_ref(&incoming))?.len() as u64 <= MAX_BYTES,
            tr(
                "Questa conversione supera lo spazio disponibile nella cronologia",
                "This conversion exceeds the available history space"
            )
        );
    }
    jobs.retain(|previous| previous.id != incoming.id);
    jobs.insert(0, incoming);
    jobs.truncate(MAX_JOBS);
    let bytes = loop {
        let bytes = encoded(&jobs)?;
        if bytes.len() as u64 <= MAX_BYTES {
            break bytes;
        }
        ensure!(
            jobs.len() > 1,
            tr(
                "La voce della cronologia supera 1 MiB",
                "The history entry exceeds 1 MiB"
            )
        );
        jobs.pop();
    };
    let mut temporary = tempfile::NamedTempFile::new_in(parent(path))?;
    the_converter::platform::set_hidden(temporary.path())?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .context(tr(
            "Impossibile salvare la cronologia",
            "Could not save history",
        ))?;
    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::{Arc, Barrier},
    };
    use the_converter::queue::JobState;

    fn job(id: &str) -> QueueJob {
        QueueJob {
            id: id.into(),
            input: PathBuf::from("input.png"),
            format: "PNG".into(),
            outputs: vec![PathBuf::from("result.png")],
            state: JobState::Completed,
            progress: 1.0,
            message: "Done".into(),
            error: None,
        }
    }

    #[test]
    fn async_history_accepts_completions_while_locked_and_refreshes_other_windows() {
        use std::time::{Duration, Instant};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.json");
        record(&path, &job("initial")).unwrap();
        let lock = open_lock(&path).unwrap();
        fs2::FileExt::lock_exclusive(&lock).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        let thread_path = path.clone();
        let thread = std::thread::spawn(move || {
            sender
                .send(record_async(thread_path, job("async")))
                .unwrap();
        });
        // This timeout detects waiting on a file lock, not encoder performance.
        let enqueued = receiver.recv_timeout(Duration::from_secs(2));
        drop(lock);
        thread.join().unwrap();
        enqueued
            .expect("enqueue must not wait for the file lock")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !load(&path).unwrap().iter().any(|job| job.id == "async") {
            assert!(
                Instant::now() < deadline,
                "background history write did not complete"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        let history = AsyncHistory::open(path.clone()).unwrap();
        record(&path, &job("other-window")).unwrap();
        history.refresh();
        loop {
            if history
                .snapshot_after(0)
                .is_some_and(|snapshot| snapshot.jobs.iter().any(|job| job.id == "other-window"))
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "external history refresh did not complete"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn close_flush_remains_responsive_and_a_second_close_can_leave_after_timeout() {
        let ctx = egui::Context::default();
        let saving = ctx.run(egui::RawInput::default(), |ctx| {
            assert!(guard_close_pending(ctx, false, true, 1).is_some_and(|(_, error)| !error));
        });
        assert!(
            saving.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|command| matches!(command, egui::ViewportCommand::CancelClose))
        );
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("gui_history_close_flush"),
                CloseFlush {
                    started: std::time::Instant::now() - std::time::Duration::from_secs(6),
                    timed_out: false,
                    allow_close: false,
                },
            )
        });
        let timeout = ctx.run(egui::RawInput::default(), |ctx| {
            assert!(guard_close_pending(ctx, false, true, 1).is_some_and(|(_, error)| error));
        });
        assert!(
            timeout.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|command| matches!(command, egui::ViewportCommand::CancelClose))
        );
        let mut input = egui::RawInput::default();
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .events
            .push(egui::ViewportEvent::Close);
        let retry = ctx.run(input, |ctx| {
            assert!(guard_close_pending(ctx, false, true, 1).is_none());
            assert!(guard_close_pending(ctx, false, true, 1).is_none());
        });
        assert!(
            !retry.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|command| matches!(command, egui::ViewportCommand::CancelClose))
        );
        let clean = egui::Context::default();
        let _ = clean.run(egui::RawInput::default(), |ctx| {
            let _ = guard_close_pending(ctx, false, true, 1);
        });
        let completed = clean.run(egui::RawInput::default(), |ctx| {
            assert!(guard_close_pending(ctx, false, false, 0).is_none());
        });
        assert!(
            completed.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|command| matches!(command, egui::ViewportCommand::Close))
        );
    }

    #[test]
    fn terminal_history_persists_deduplicates_and_never_replaces_corrupt_data() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.json");
        assert!(load(&path).unwrap().is_empty());
        let mut first = job("first");
        first.state = JobState::Running;
        assert!(record(&path, &first).is_err());
        assert!(!path.exists());
        first.state = JobState::Completed;
        record(&path, &first).unwrap();
        record(&path, &job("second")).unwrap();
        first.state = JobState::Failed;
        first.error = Some("Actual encoder error".into());
        record(&path, &first).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(
            loaded.iter().map(|job| job.id.as_str()).collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert_eq!(loaded[0].error, first.error);
        let valid = std::fs::read(&path).unwrap();
        let mut unsupported: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        unsupported["version"] = serde_json::json!(99);
        for invalid in [
            b"{broken".to_vec(),
            serde_json::to_vec(&unsupported).unwrap(),
            vec![b' '; MAX_BYTES as usize + 1],
        ] {
            std::fs::write(&path, &invalid).unwrap();
            assert!(load(&path).is_err());
            assert!(record(&path, &job("third")).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), invalid);
        }
    }

    #[test]
    fn concurrent_readers_and_writers_keep_valid_bounded_history_and_large_output_lists() {
        let directory = tempfile::tempdir().unwrap();
        let path = Arc::new(directory.path().join("history.json"));
        let barrier = Arc::new(Barrier::new(5));
        let mut threads = Vec::new();
        for writer in 0..4 {
            let path = path.clone();
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                for index in 0..80 {
                    record(&path, &job(&format!("{writer}-{index}"))).unwrap();
                }
            }));
        }
        barrier.wait();
        for _ in 0..40 {
            let snapshot = load(&path).unwrap();
            assert!(snapshot.len() <= MAX_JOBS);
            assert_eq!(
                snapshot
                    .iter()
                    .map(|job| &job.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                snapshot.len()
            );
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(load(&path).unwrap().len(), MAX_JOBS);
        let mut huge = job("huge");
        huge.outputs = (0..4000)
            .map(|index| PathBuf::from(format!("{index}-{}.pdf", "x".repeat(400))))
            .collect();
        let snapshot = record(&path, &huge).unwrap();
        assert_eq!(snapshot[0].id, "huge");
        assert_eq!(snapshot[0].outputs.len(), 32);
        assert_eq!(snapshot[0].outputs[0], huge.outputs[0]);
        assert!(snapshot[0].message.contains("4000"));
        let mut many = job("many");
        many.outputs = (0..1000)
            .map(|index| PathBuf::from(format!("{index}-{}.pdf", "x".repeat(100))))
            .collect();
        record(&path, &many).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded[0].outputs, many.outputs);
        assert!(std::fs::metadata(&*path).unwrap().len() <= MAX_BYTES);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2); // JSON + stable lock only.
    }
}

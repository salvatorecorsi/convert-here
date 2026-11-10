//! Monotonic, per-user acknowledgement of the optional engine setup wizard.
//! Stored separately so another open window cannot undo it with stale preferences.
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

const MARKER_NAME: &str = "engine-setup-v1.done";
const MARKER_CONTENTS: &[u8] = b"Convert Here engine setup v1\n";

fn marker_path(settings_path: &Path) -> io::Result<PathBuf> {
    if settings_path.file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Preferences path must name a file",
        ));
    }
    Ok(settings_path.with_file_name(MARKER_NAME))
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false)
}

pub(crate) fn was_seen(settings_path: Option<&Path>) -> bool {
    settings_path
        .and_then(|path| marker_path(path).ok())
        .is_some_and(|path| is_regular_file(&path))
}

/// Acknowledge completion or an explicit skip without changing preferences.
/// A concurrent acknowledgement or an existing regular marker is successful.
pub(crate) fn mark_seen(settings_path: &Path) -> io::Result<()> {
    let marker = marker_path(settings_path)?;
    if is_regular_file(&marker) {
        return Ok(());
    }
    let parent = marker
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(MARKER_CONTENTS)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    match temporary.persist_noclobber(&marker) {
        Ok(_) => Ok(()),
        Err(error)
            if error.error.kind() == io::ErrorKind::AlreadyExists && is_regular_file(&marker) =>
        {
            Ok(())
        }
        Err(error) => Err(error.error),
    }
}

pub(crate) fn should_offer(custom: bool, has_files: bool, has_target: bool, seen: bool) -> bool {
    !custom && !has_files && !has_target && !seen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_marker_and_legacy_preferences_have_not_seen_setup() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("preferences.json");
        assert!(!was_seen(None));
        assert!(!was_seen(Some(&settings)));
        fs::write(&settings, br#"{"preset":3,"settings":{"format":"Flac"}}"#).unwrap();
        assert!(!was_seen(Some(&settings)));
        assert!(!was_seen(Some(Path::new(""))));
    }

    #[test]
    fn acknowledgement_survives_stale_preferences_without_replacing_a_marker() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("preferences.json");
        let original = br#"{"preset":3,"settings":{"format":"Flac"}}"#;
        fs::write(&settings, original).unwrap();
        mark_seen(&settings).unwrap();
        let marker = marker_path(&settings).unwrap();
        assert_eq!(fs::read(&marker).unwrap(), MARKER_CONTENTS);
        assert_eq!(fs::read(&settings).unwrap(), original);
        assert!(was_seen(Some(&settings)));

        // Another GUI may rewrite an older settings snapshot after setup.
        fs::write(&settings, br#"{"preset":1}"#).unwrap();
        assert!(was_seen(Some(&settings)));
        fs::write(&marker, b"existing regular marker").unwrap();
        mark_seen(&settings).unwrap();
        assert_eq!(fs::read(&marker).unwrap(), b"existing regular marker");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn first_acknowledgement_creates_only_its_marker_in_a_new_config_folder() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory
            .path()
            .join("new")
            .join("config")
            .join("preferences.json");
        mark_seen(&settings).unwrap();
        assert!(was_seen(Some(&settings)));
        assert!(!settings.exists());
        assert_eq!(fs::read_dir(settings.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn directory_collision_and_blocked_parent_fail_without_changing_existing_data() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("preferences.json");
        let marker = marker_path(&settings).unwrap();
        fs::create_dir(&marker).unwrap();
        let existing = marker.join("keep.txt");
        fs::write(&existing, b"keep this data").unwrap();
        assert!(!was_seen(Some(&settings)));
        assert!(mark_seen(&settings).is_err());
        assert_eq!(fs::read(&existing).unwrap(), b"keep this data");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);

        let blocked = directory.path().join("blocked");
        fs::write(&blocked, b"keep this file").unwrap();
        assert!(mark_seen(&blocked.join("preferences.json")).is_err());
        assert_eq!(fs::read(&blocked).unwrap(), b"keep this file");
        assert!(mark_seen(Path::new("")).is_err());
    }

    #[test]
    fn concurrent_acknowledgements_publish_one_complete_marker() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("preferences.json");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let settings = settings.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    mark_seen(&settings)
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap().unwrap();
        }
        assert!(was_seen(Some(&settings)));
        assert_eq!(
            fs::read(marker_path(&settings).unwrap()).unwrap(),
            MARKER_CONTENTS
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn setup_is_offered_only_for_an_empty_normal_gui_not_previously_acknowledged() {
        for custom in [false, true] {
            for has_files in [false, true] {
                for has_target in [false, true] {
                    for seen in [false, true] {
                        assert_eq!(
                            should_offer(custom, has_files, has_target, seen),
                            (custom, has_files, has_target, seen) == (false, false, false, false)
                        );
                    }
                }
            }
        }
    }
}

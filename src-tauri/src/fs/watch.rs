use chrono::{ DateTime, Utc };
use futures::{ channel::mpsc::{ channel, Receiver }, SinkExt, StreamExt };
use notify::{
    Config,
    Event,
    EventKind,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
    event::{ ModifyKind, RenameMode },
};
use std::{ fs, path::{ Path, PathBuf } };
use tauri::{ AppHandle, Emitter };

use crate::db::{ self, screenshot_schema::{ Model, OptionalModel } };

fn get_file_details(file_path: PathBuf) -> Result<Model, String> {
    let metadata = fs
        ::metadata(&file_path)
        .map_err(|e| format!("Cannot read metadata for {}: {e}", file_path.display()))?;

    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Cannot get a valid UTF-8 file name".to_string())?
        .to_string();

    let date = metadata.created().map_err(|e| format!("Cannot read creation time: {e}"))?;

    let date_utc: DateTime<Utc> = date.into();

    Ok(Model {
        name,
        date: date_utc,
        path: file_path.to_string_lossy().to_string(),
    })
}

fn file_name_string(p: &Path) -> Option<String> {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned)
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            if let Err(e) = tx.send(res).await {
                tracing::error!("WATCH ERROR: could not forward event, receiver closed: {e}");
            }
        })
    }, Config::default())?;

    Ok((watcher, rx))
}

// ^ Tracking the rename events
#[derive(Default)]
struct RenameTracker {
    pending_from: Option<PathBuf>,
}

impl RenameTracker {
    fn on_from(&mut self, paths: &[PathBuf]) {
        match paths {
            [] => {}
            [p] => {
                if let Some(stale) = self.pending_from.replace(p.clone()) {
                    tracing::warn!("Name(From) {p:?} replaced an unpaired pending From: {stale:?}");
                }
            }
            many => {
                tracing::error!("Modify(Name(From)) with more than 1 path: {many:?}");
                self.pending_from = None; // don't leave stale state behind
            }
        }
    }

    /// Returns (old, new) when this `To` completes a pair.
    fn on_to(&mut self, paths: &[PathBuf]) -> Option<(PathBuf, PathBuf)> {
        match paths {
            [] => None,
            [new] =>
                match self.pending_from.take() {
                    Some(old) => Some((old, new.clone())),
                    None => {
                        tracing::error!(
                            "Received Modify(Name(To)) Event but no Name(From) event found."
                        );
                        None
                    }
                }
            many => {
                self.pending_from = None;
                tracing::error!("Modify(Name(To)) with more than 1 path: {many:?}");
                None
            }
        }
    }
}

// ^ helper func for a rename event
async fn rename_entry(old_path: &Path, new_path: &Path) {
    // Windows can't tell us the kind on From, but the To path exists, so check it here
    if new_path.is_dir() {
        tracing::debug!("Ignoring rename of directory: {old_path:?} -> {new_path:?}");
        return;
    }

    let Some(new_name) = file_name_string(new_path) else {
        tracing::error!("New path has no valid UTF-8 file name: {new_path:?}");
        return;
    };
    let Some(old_name) = file_name_string(old_path) else {
        tracing::error!("Old path has no valid UTF-8 file name: {old_path:?}");
        return;
    };

    let old_path_str = old_path.to_string_lossy().to_string();
    let new_path_str = new_path.to_string_lossy().to_string();

    let modify_obj = OptionalModel {
        path: Some(new_path_str.clone()),
        name: Some(new_name),
        date: None,
    };

    match db::commands::update_entry(old_name, modify_obj).await {
        Ok(true) => tracing::warn!("Updated {old_path_str} to {new_path_str}"),
        Ok(false) =>
            tracing::error!(
                "WATCH x DB ERROR when applying update event: {old_path_str} does not exist"
            ),
        Err(e) => tracing::error!("WATCH x DB ERROR when applying update event: {e}"),
    }
}

// ^ Function handling all the events for watch
pub async fn async_watch<P: AsRef<Path>>(app: AppHandle, path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    // handler for From/To events
    let mut tracker = RenameTracker::default();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                // * handle each event
                match event.kind {
                    EventKind::Create(_) => {
                        for path in event.paths {
                            let new_entry = match get_file_details(path) {
                                Ok(r) => r,
                                Err(e) => {
                                    tracing::error!(
                                        "WATCH x FS ERROR when getting file details: {e}"
                                    );
                                    continue;
                                }
                            };
                            let _ = db::commands::insert_entry(new_entry).await.map_err(|e| {
                                tracing::error!("WATCH x DB ERROR when applying insert event: {e}");
                            });
                            tracing::warn!("Created");
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::From)) =>
                        tracker.on_from(&event.paths),
                    EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        if let Some((old, new)) = tracker.on_to(&event.paths) {
                            rename_entry(&old, &new).await;
                        }
                    }
                    EventKind::Modify(other) => {
                        tracing::debug!(
                            "WATCH: unhandled Modify kind: {other:?} with full event data: {event:?}"
                        );
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            let Some(file_name) = file_name_string(&path) else {
                                tracing::error!("WATCH ERROR File Name not found");
                                continue;
                            };
                            match db::commands::delete_entry(file_name.clone()).await {
                                Ok(true) => tracing::warn!("Deleted"),
                                Ok(false) =>
                                    tracing::error!(
                                        "WATCH x DB ERROR when applying delete event: {file_name} does not exist"
                                    ),
                                Err(e) =>
                                    tracing::error!(
                                        "WATCH x DB ERROR when applying delete event: {e}"
                                    ),
                            }
                        }
                    }
                    EventKind::Access(_) => {}
                    EventKind::Any | EventKind::Other => {
                        tracing::warn!("WATCH WARN - UNKNOWN EVENT: {event:?}");
                    }
                }
            }
            Err(e) => tracing::error!("WATCH ERROR: {e:?}"),
        }
    }

    Ok(())
}

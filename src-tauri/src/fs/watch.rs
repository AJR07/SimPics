use chrono::{DateTime, Utc};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Emitter};

use crate::db::{self, screenshot_schema::Model};

fn get_file_details(file_path: PathBuf) -> Result<Model, String> {
    let metadata = fs::metadata(&file_path)
        .map_err(|e| format!("Cannot read metadata for {}: {e}", file_path.display()))?;

    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Cannot get a valid UTF-8 file name".to_string())?
        .to_string();

    let date = metadata
        .created()
        .map_err(|e| format!("Cannot read creation time: {e}"))?;

    let date_utc: DateTime<Utc> = date.into();

    Ok(Model {
        name,
        date: date_utc,
        path: file_path.to_string_lossy().to_string()
    })
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn async_watch<P: AsRef<Path>>(app: AppHandle, path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                app.emit("file-watcher_change", format!("changed: {event:?}"))
                    .unwrap();

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
                                tracing::error!(
                                    "WATCH x DB ERROR when applying insert event: {e}"
                                );
                            });
                            tracing::warn!("Created");
                        }
                    }
                    EventKind::Modify(_) => {}
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                                Some(r) => r.to_string(),
                                None => {
                                    tracing::error!(
                                        "WATCH x FS ERROR when getting file details: No file_name found"
                                    );
                                    continue;
                                }
                            };
                            match db::commands::delete_entry(file_name.clone()).await {
                                Ok(true) => continue,
                                Ok(false) => tracing::error!(
                                    "WATCH x DB ERROR when applying delete event: {file_name} does not exist"
                                ),
                                Err(e) => 
                                tracing::error!(
                                    "WATCH x DB ERROR when applying delete event event: {e}"
                                )
                            }
                            tracing::warn!("Deleted");
                        }
                    }
                    EventKind::Access(_) => {}
                    EventKind::Any | EventKind::Other => {
                        tracing::warn!("WATCH WARN - UNKNOWN EVENT: {event:?}");
                    }
                }
            }
            Err(e) => {
                app.emit("file-watcher_error", format!("watch error: {e:?}"))
                    .unwrap();
                tracing::error!("WATCH ERROR: {e:?}")
            }
        }
    }

    Ok(())
}

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
            tx.send(res).await.unwrap();
        })
    }, Config::default())?;

    Ok((watcher, rx))
}

pub async fn async_watch<P: AsRef<Path>>(app: AppHandle, path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    // handler for From/To events
    let mut prev_modify: Option<PathBuf> = None::<PathBuf>;

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
                    EventKind::Modify(e) => {
                        // filter for events of Name(From) and Name(To)
                        match e {
                            ModifyKind::Name(RenameMode::From) => {
                                match event.paths.len() {
                                    0 => {}
                                    1 => {
                                        prev_modify = Some(event.paths[0].clone());
                                    }
                                    _ => {
                                        tracing::error!(
                                            "Event {e:?} with more than 1 path for Modify(Name(From))"
                                        );
                                    }
                                }
                            }
                            ModifyKind::Name(RenameMode::To) => {
                                match event.paths.len() {
                                    0 => {}
                                    1 => {
                                        if let None = prev_modify {
                                            tracing::error!(
                                                "Received Modify(Name(To)) Event but no Name(From) event found."
                                            );
                                        } else {
                                            // * perform the update with the previous value
                                            let Some(new_path) = event.paths.first().cloned() else {
                                                continue;
                                            };

                                            let new_path_str = new_path
                                                .to_string_lossy()
                                                .to_string();

                                            let Some(new_name) = file_name_string(&new_path) else {
                                                tracing::error!(
                                                    "New path has no valid UTF-8 file name: {new_path:?}"
                                                );
                                                continue;
                                            };

                                            // take() clears the pending state so it can't be reused by a later event
                                            let Some(old_path) = prev_modify.take() else {
                                                tracing::error!(
                                                    "Received Modify(Name(To)) but no Name(From) was pending"
                                                );
                                                continue;
                                            };

                                            let old_path_str = old_path
                                                .to_string_lossy()
                                                .to_string();

                                            let Some(old_name) = file_name_string(&old_path) else {
                                                tracing::error!(
                                                    "Old path has no valid UTF-8 file name: {old_path:?}"
                                                );
                                                continue;
                                            };

                                            // craft the modify object
                                            let modify_obj = OptionalModel {
                                                path: Some(new_path_str.clone()),
                                                name: Some(new_name),
                                                date: None,
                                            };

                                            // finally, update
                                            match
                                                db::commands::update_entry(
                                                    old_name,
                                                    modify_obj
                                                ).await
                                            {
                                                Ok(true) =>
                                                    tracing::warn!(
                                                        "Updated {old_path_str} to {new_path_str}"
                                                    ),
                                                Ok(false) =>
                                                    tracing::error!(
                                                        "WATCH x DB ERROR when applying update event: {old_path_str} does not exist"
                                                    ),
                                                Err(e) =>
                                                    tracing::error!(
                                                        "WATCH x DB ERROR when applying update event: {e}"
                                                    ),
                                            };
                                        }
                                    }
                                    _ => {
                                        tracing::error!(
                                            "Event {e:?} with more than 1 path for Modify(Name(To))"
                                        );
                                    }
                                }
                            }
                            _ => {
                                tracing::debug!(
                                    "WATCH ERROR: Unhandled Event Kind: {e:?} with full event data: {event:?}"
                                );
                            }
                        }
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
            Err(e) => {
                app.emit("file-watcher_error", format!("watch error: {e:?}")).unwrap();
                tracing::error!("WATCH ERROR: {e:?}");
            }
        }
    }

    Ok(())
}

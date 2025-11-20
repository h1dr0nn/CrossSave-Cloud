use std::{collections::HashMap, path::PathBuf, pin::Pin, sync::Mutex, time::Duration};

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use serde::Serialize;
use thiserror::Error;
use tokio::select;
use tokio::time::{sleep, Instant, Sleep};
use tracing::{debug, error, info, warn};

use tauri::{AppHandle, Emitter};

const DEFAULT_DEBOUNCE_MS: u64 = 200;
const WATCHER_EVENT_NAME: &str = "watcher://fs-event";

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("watcher already running")]
    AlreadyRunning,
    #[error("no watcher is currently running")]
    NotRunning,
    #[error("failed to acquire watcher lock: {0}")]
    Lock(String),
    #[error("failed to create watcher: {0}")]
    Create(String),
    #[error("failed to watch path {0}: {1}")]
    WatchPath(String, String),
}

#[derive(Debug)]
struct WatcherInstance {
    watcher: RecommendedWatcher,
    stop_tx: async_channel::Sender<()>,
    task_handle: tauri::async_runtime::JoinHandle<()>,
}

#[derive(Clone, Debug, Serialize)]
pub enum WatchEventType {
    Add,
    Modify,
    Delete,
}

#[derive(Clone, Debug, Serialize)]
pub struct WatchEventPayload {
    pub path: PathBuf,
    pub event_type: WatchEventType,
}

#[derive(Default)]
pub struct WatcherManager {
    inner: Mutex<Option<WatcherInstance>>,
}

impl WatcherManager {
    pub fn start(&self, app: AppHandle, paths: Vec<PathBuf>) -> Result<(), WatcherError> {
        if paths.is_empty() {
            return Err(WatcherError::WatchPath(
                "<empty>".into(),
                "no paths provided".into(),
            ));
        }

        let mut guard = self
            .inner
            .lock()
            .map_err(|err| WatcherError::Lock(err.to_string()))?;
        if guard.is_some() {
            warn!("[WATCHER] Attempted to start watcher while already running");
            return Err(WatcherError::AlreadyRunning);
        }

        let filtered_paths: Vec<PathBuf> = paths
            .into_iter()
            .filter(|path| {
                if path.exists() {
                    true
                } else {
                    warn!("[WATCHER] Skipping non-existent path: {:?}", path);
                    false
                }
            })
            .collect();

        if filtered_paths.is_empty() {
            return Err(WatcherError::WatchPath(
                "<empty>".into(),
                "no valid paths".into(),
            ));
        }

        info!(
            "[WATCHER] Starting watcher for {} paths",
            filtered_paths.len()
        );

        let (event_tx, event_rx) = async_channel::unbounded::<NotifyResult<Event>>();
        let (stop_tx, stop_rx) = async_channel::bounded::<()>(1);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(err) = event_tx.try_send(res) {
                    warn!("[WATCHER] Dropped event due to channel error: {err}");
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(DEFAULT_DEBOUNCE_MS)),
        )
        .map_err(|err| WatcherError::Create(err.to_string()))?;

        for path in &filtered_paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|err| {
                    WatcherError::WatchPath(path.display().to_string(), err.to_string())
                })?;
            debug!("[WATCHER] Watching path: {:?}", path);
        }

        let handle = spawn_processor(
            app.clone(),
            stop_rx,
            event_rx,
            Duration::from_millis(DEFAULT_DEBOUNCE_MS),
        );

        *guard = Some(WatcherInstance {
            watcher,
            stop_tx,
            task_handle: handle,
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), WatcherError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|err| WatcherError::Lock(err.to_string()))?;
        let Some(instance) = guard.take() else {
            warn!("[WATCHER] Attempted to stop watcher but none running");
            return Err(WatcherError::NotRunning);
        };

        info!("[WATCHER] Stopping watcher");
        let _ = instance.stop_tx.send(()).await;
        instance.task_handle.abort();
        Ok(())
    }
}

fn spawn_processor(
    app: AppHandle,
    stop_rx: async_channel::Receiver<()>,
    event_rx: async_channel::Receiver<NotifyResult<Event>>,
    debounce: Duration,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut pending_events: HashMap<PathBuf, WatchEventType> = HashMap::new();
        let mut debounce_timer: Pin<Box<Sleep>> = Box::pin(sleep(debounce));

        loop {
            select! {
                _ = stop_rx.recv() => {
                    debug!("[WATCHER] Received stop signal");
                    break;
                }
                maybe_event = event_rx.recv() => {
                    match maybe_event {
                        Ok(Ok(event)) => {
                            if register_event(&mut pending_events, &event) {
                                debounce_timer.as_mut().reset(Instant::now() + debounce);
                            }
                        }
                        Ok(Err(err)) => error!("[WATCHER] Error from watcher: {err}"),
                        Err(_) => break,
                    }
                }
                _ = &mut debounce_timer => {
                    flush_events(&app, &mut pending_events).await;
                    debounce_timer.as_mut().reset(Instant::now() + debounce);
                }
            }
        }

        flush_events(&app, &mut pending_events).await;
        info!("[WATCHER] Watcher processor stopped");
    })
}

fn register_event(pending: &mut HashMap<PathBuf, WatchEventType>, event: &Event) -> bool {
    let event_type = match map_event_kind(&event.kind) {
        Some(kind) => kind,
        None => return false,
    };

    let mut registered = false;
    for path in &event.paths {
        pending.insert(path.clone(), event_type.clone());
        registered = true;
    }

    registered
}

async fn flush_events(app: &AppHandle, pending: &mut HashMap<PathBuf, WatchEventType>) {
    if pending.is_empty() {
        return;
    }

    let events: Vec<WatchEventPayload> = pending
        .drain()
        .map(|(path, event_type)| WatchEventPayload { path, event_type })
        .collect();

    for event in events {
        debug!(
            "[WATCHER] Emitting {:?} for {:?}",
            event.event_type, event.path
        );
        if let Err(err) = app.emit(WATCHER_EVENT_NAME, &event) {
            error!("[WATCHER] Failed to emit event: {err}");
        }
    }
}

fn map_event_kind(kind: &EventKind) -> Option<WatchEventType> {
    use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};

    match kind {
        EventKind::Create(CreateKind::Any)
        | EventKind::Create(CreateKind::File)
        | EventKind::Create(CreateKind::Folder)
        | EventKind::Create(CreateKind::Other) => Some(WatchEventType::Add),
        EventKind::Modify(ModifyKind::Any)
        | EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Metadata(_))
        | EventKind::Modify(ModifyKind::Name(RenameMode::Both))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To))
        | EventKind::Modify(ModifyKind::Other) => Some(WatchEventType::Modify),
        EventKind::Remove(RemoveKind::Any)
        | EventKind::Remove(RemoveKind::File)
        | EventKind::Remove(RemoveKind::Folder)
        | EventKind::Remove(RemoveKind::Other)
        | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => Some(WatchEventType::Delete),
        _ => None,
    }
}

pub fn watcher_event_name() -> &'static str {
    WATCHER_EVENT_NAME
}

pub type SharedWatcherManager = tauri::State<'_, WatcherManager>;

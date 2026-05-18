use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::registry::ToolRegistry;
use crate::tools::ExternalTool;

pub fn spawn_watcher(
    tools_dir: PathBuf,
    registry: Arc<RwLock<ToolRegistry>>,
    timeout_secs: u64,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default().with_poll_interval(Duration::from_secs(1)),
    )
    .expect("failed to create watcher");

    watcher
        .watch(&tools_dir, RecursiveMode::NonRecursive)
        .expect("failed to watch tools_dir");

    tokio::spawn(async move {
        let _watcher = watcher;
        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => handle_event(event, &tools_dir, &registry, timeout_secs),
                Err(e) => warn!("watcher error: {}", e),
            }
        }
    });
}

fn handle_event(
    event: Event,
    tools_dir: &PathBuf,
    registry: &Arc<RwLock<ToolRegistry>>,
    timeout_secs: u64,
) {
    for path in &event.paths {
        let parent = path.parent();
        if parent != Some(tools_dir.as_path()) {
            continue;
        }
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        match event.kind {
            EventKind::Create(_) => {
                let mut reg = registry.write().unwrap();
                if reg.get(&name).is_none() {
                    reg.register(Arc::new(ExternalTool::new(&name, path.clone(), timeout_secs)));
                    info!("hot-reload: registered tool '{}'", name);
                }
            }
            EventKind::Remove(_) => {
                let mut reg = registry.write().unwrap();
                reg.unregister(&name);
                info!("hot-reload: unregistered tool '{}'", name);
            }
            _ => {}
        }
    }
}

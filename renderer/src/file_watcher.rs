use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{info, warn};

use crate::resource_store::reload_command::RebuildCommand;

pub struct FileWatcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    watched_files: HashMap<PathBuf, RebuildCommand>,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(project_root: P) -> eyre::Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(project_root.as_ref(), RecursiveMode::Recursive)?;
        Ok(Self {
            watcher,
            receiver: rx,
            watched_files: HashMap::new(),
        })
    }

    pub fn process_updates(&mut self) -> Vec<RebuildCommand> {
        let mut commands = Vec::new();
        for message in self.receiver.try_iter() {
            match message {
                Ok(message) => {
                    for changed_path in message.paths {
                        info!("File changed: {}", changed_path.display());
                        if let Some(command) = self.watched_files.get(&changed_path) {
                            commands.push(command.clone());
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        "File reloading may be broken; the file watcher failed with: {}",
                        err
                    );
                }
            }
        }
        commands
    }

    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P, watched_object: RebuildCommand) {
        self.watched_files
            .insert(path.as_ref().to_path_buf(), watched_object);
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::warn;

pub enum UpdateCommand {
    Keep,
    Remove,
}

pub trait WatchedResource {
    fn update(&mut self) -> UpdateCommand;
}

pub struct ResourceWatcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    watched_files: HashMap<PathBuf, Box<dyn WatchedResource>>,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl ResourceWatcher {
    pub fn new<P: AsRef<Path>>(project_root: P) -> eyre::Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(project_root.as_ref(), RecursiveMode::Recursive)?;
        Ok(Self {
            watcher,
            watched_files: HashMap::new(),
            receiver: rx,
        })
    }

    pub fn process_updates(&mut self) {
        for message in self.receiver.try_iter() {
            match message {
                Ok(message) => {
                    for changed_path in message.paths {
                        let pointer = self.watched_files.get_mut(&changed_path);
                        if let Some(pointer) = pointer {
                            if let UpdateCommand::Remove = pointer.update() {
                                self.watched_files.remove(&changed_path);
                            }
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
    }

    pub fn watch_file<P: AsRef<Path>>(
        &mut self,
        path_sufix: P,
        resource: Box<dyn WatchedResource>,
    ) {
        self.watched_files
            .insert(path_sufix.as_ref().to_path_buf(), resource);
    }
}

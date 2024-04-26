use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::warn;

pub trait ResourceChangeObserver {
    fn file_changed(&self);
}

pub struct ResourceWatcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    watched_files: HashMap<PathBuf, Box<dyn ResourceChangeObserver>>,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl ResourceWatcher {
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

    pub fn process_updates(&mut self) {
        for message in self.receiver.try_iter() {
            match message {
                Ok(message) => {
                    for changed_path in message.paths {
                        println!("some changes: {}", changed_path.display());
                        if let Some(watched_object) = self.watched_files.get_mut(&changed_path) {
                            watched_object.file_changed();
                        }
                    }
                    println!("this sucksssss....: {:?}", self.watched_files.keys());
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
        watched_object: Box<dyn ResourceChangeObserver>,
    ) {
        self.watched_files
            .insert(path_sufix.as_ref().to_path_buf(), watched_object);
    }
}

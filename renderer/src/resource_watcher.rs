use std::{collections::HashMap, env, path::{Path, PathBuf}, sync::mpsc::{channel, Receiver}};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::warn;

use crate::context::Context;

pub trait Reloadable {
    fn reload(&mut self, context: &Context);
}

pub struct ResourceWatcher {
    resource_map: HashMap<PathBuf, Box<dyn Reloadable>>,
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl ResourceWatcher {
    pub fn new() -> ResourceWatcher {
            let (tx, rx) = channel();
            let pwd = env::current_dir().unwrap();
            let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
            watcher.watch(pwd.as_ref(), RecursiveMode::Recursive).unwrap();
            ResourceWatcher{
                resource_map: HashMap::default(),
                watcher,
                receiver: rx,
            }
    }

    pub fn watch_resource<P: AsRef<Path>>(&mut self, path: P, resource: Box<dyn Reloadable>) {
        let pwd = env::current_dir().unwrap();
        let path = pwd.join(path).canonicalize().unwrap();
        self.resource_map.insert(path, resource);
    }

    pub fn process_watcher_updates(&mut self, context: &Context) {
        for message in self.receiver.try_iter() {
            match message {
                Ok(message) => {
                    for changed_path in message.paths {
                        println!("some changes: {}", changed_path.display());
                        if let Some(watched_object) = self.resource_map.get_mut(&changed_path) {
                            watched_object.reload(context);
                        }
                    }
                    println!("this sucksssss....: {:?}", self.resource_map.keys());
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
}

impl Default for ResourceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

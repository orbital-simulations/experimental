use std::rc::Weak;

use notify::INotifyWatcher;

pub trait WatchedResource {
    fn update(&mut self);
}

struct ResourceWatcher {
    watcher: INotifyWatcher,
    watched_files: Vec<Weak<dyn WatchedResource>>,
}

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Default, Clone)]
pub struct ThreadManager {
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_thread<T>(&self, handle: JoinHandle<T>)
    where
        T: Send + 'static,
    {
        let wrapped_handle = std::thread::spawn(move || {
            handle.join().expect("Thread panicked");
        });
        self.handles.lock().unwrap().push(wrapped_handle);
    }

    pub fn join_all(&self) {
        let mut handles = self.handles.lock().unwrap();
        while let Some(handle) = handles.pop() {
            handle.join().expect("Thread failed to join");
        }
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Per-mount session state shared across all tool invocations.
#[derive(Clone, Default)]
pub struct Session {
    #[allow(dead_code)]
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.state.lock().unwrap().get(key).cloned()
    }

    #[allow(dead_code)]
    pub fn set(&self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.state.lock().unwrap().insert(key.into(), value.into());
    }

    #[allow(dead_code)]
    pub fn remove(&self, key: &str) {
        self.state.lock().unwrap().remove(key);
    }
}

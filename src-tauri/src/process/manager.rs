use std::collections::HashMap;

use crate::models::session::SessionId;

pub struct ProcessManager {
    _processes: HashMap<SessionId, ()>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            _processes: HashMap::new(),
        }
    }

    pub fn session_count(&self) -> usize {
        self._processes.len()
    }
}

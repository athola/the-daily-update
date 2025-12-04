//! Application state and main event loop

pub struct App {
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self { running: true }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

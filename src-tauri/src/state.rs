use crate::db;
use crate::error::Result;
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub cancel_scan: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let conn = db::open()?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            cancel_scan: Arc::new(AtomicBool::new(false)),
        })
    }
}

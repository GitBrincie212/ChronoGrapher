use std::path::{Path};
use rocksdb::{Options, DBWithThreadMode, MultiThreaded};

pub struct DefaultPersistenceBackend {
    db: DBWithThreadMode<MultiThreaded>,
}

impl DefaultPersistenceBackend {
    pub fn new(snapshot_path: impl AsRef<Path>) -> Self {
        let options = Options::default();
        let db: DBWithThreadMode<MultiThreaded> = DBWithThreadMode::open(&options, snapshot_path).unwrap();
        Self { db }
    }
}
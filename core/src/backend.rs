pub mod default;

pub use default::*;
use serde_json::json;

pub trait PersistenceBackend: Send + Sync {
    fn serialize(&self, obj: serde_json::Value);
    fn deserialize(&self, bytes: Box<[u8]>) -> serde_json::Value;
}

impl PersistenceBackend for () {
    fn serialize(&self, _obj: serde_json::Value) {}

    fn deserialize(&self, _bytes: Box<[u8]>) -> serde_json::Value {
        json!({})
    }
}
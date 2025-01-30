use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TextMessage {
    pub serial: u64,
    pub subseq: u64,
    pub event: String,
    data: Option<HashMap<String, serde_json::Value>>,
}

impl TextMessage {
    pub fn new(serial: u64, subseq: u64, event: String) -> Self {
        TextMessage {
            serial,
            subseq,
            event,
            data: None,
        }
    }

    pub fn next(&self, event: String) -> Self {
        TextMessage {
            serial: self.serial,
            subseq: self.subseq + 1,
            event,
            data: None,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn get_data(&self, key: &str) -> Result<&Value, ()> {
        if let Some(map) = &self.data {
            if let Some(v) = map.get(key) {
                return Ok(v);
            }
        }
        Err(())
    }
    pub fn get_str(&self, key: &str) -> Result<String, ()> {
        if let Value::String(v) = self.get_data(key)? {
            Ok(v.clone())
        } else {
            Err(())
        }
    }
    pub fn get_int(self, key: &str) -> Result<i64, ()> {
        if let Value::Number(v) = self.get_data(key)? {
            v.as_i64().ok_or(())
        } else {
            Err(())
        }
    }
    pub fn get_bool(self, key: &str) -> Result<bool, ()> {
        if let Value::Bool(v) = self.get_data(key)? {
            Ok(*v)
        } else {
            Err(())
        }
    }
}
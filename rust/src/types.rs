use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Untagged so JSON values round-trip naturally: null / bool / number / string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

/// A structured object (row) in the tree. Cells are keyed by attribute id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub cells: HashMap<String, Value>,
}

/// Defines a column/attribute (its id, display label, and data kind).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: String,
    pub label: String,
    pub kind: AttributeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeKind {
    Text,
    Number,
    Checkbox,
}

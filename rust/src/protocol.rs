use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: u32,
    pub label: String,
    pub parent_id: Option<u32>,
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    GetRow(u32),
    ListRows { parent_id: Option<u32> },
    InsertRow { label: String, parent_id: Option<u32>, fields: Vec<String> },
    UpdateRow { id: u32, label: String, fields: Vec<String> },
    DeleteRow(u32),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Response {
    Row(Option<Row>),
    Rows(Vec<Row>),
    Inserted(u32),
    Updated,
    Deleted,
    Error(String),
}

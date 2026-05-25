use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use crate::protocol::{Request, Response, Row};

#[wasm_bindgen]
pub struct Server {
    rows: HashMap<u32, Row>,
    next_id: u32,
}

// Internal Rust API — takes/returns protocol types, not visible to JS.
impl Server {
    pub fn new() -> Self {
        Self {
            rows: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn handle(&mut self, req: Request) -> Response {
        match req {
            Request::GetRow(id) => Response::Row(self.rows.get(&id).cloned()),
            Request::ListRows { parent_id } => {
                let mut rows: Vec<Row> = self.rows.values()
                    .filter(|r| r.parent_id == parent_id)
                    .cloned()
                    .collect();
                rows.sort_by_key(|r| r.id);
                Response::Rows(rows)
            }
            Request::InsertRow { label, parent_id, fields } => {
                let id = self.next_id;
                self.next_id += 1;
                self.rows.insert(id, Row { id, label, parent_id, fields });
                Response::Inserted(id)
            }
            Request::UpdateRow { id, label, fields } => {
                match self.rows.get_mut(&id) {
                    Some(row) => {
                        row.label = label;
                        row.fields = fields;
                        Response::Updated
                    }
                    None => Response::Error(format!("row {} not found", id)),
                }
            }
            Request::DeleteRow(id) => {
                self.rows.remove(&id);
                Response::Deleted
            }
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

// JS/WASM API — individual methods with plain types that cross the WASM boundary.
#[wasm_bindgen]
impl Server {
    #[wasm_bindgen(constructor)]
    pub fn create() -> Server {
        Server::new()
    }

    /// Returns the new row's id.
    pub fn insert_row(&mut self, label: &str, parent_id: Option<u32>, fields: JsValue) -> u32 {
        let fields: Vec<String> = serde_wasm_bindgen::from_value(fields).unwrap_or_default();
        match self.handle(Request::InsertRow { label: label.to_string(), parent_id, fields }) {
            Response::Inserted(id) => id,
            _ => unreachable!(),
        }
    }

    /// Returns the row as a JS object, or `undefined` if not found.
    pub fn get_row(&self, id: u32) -> JsValue {
        match self.rows.get(&id) {
            Some(row) => serde_wasm_bindgen::to_value(row).unwrap_or(JsValue::UNDEFINED),
            None => JsValue::UNDEFINED,
        }
    }

    /// Returns an array of row objects whose parent_id matches. Pass `undefined` for root rows.
    pub fn list_rows(&self, parent_id: Option<u32>) -> JsValue {
        let mut rows: Vec<&Row> = self.rows.values()
            .filter(|r| r.parent_id == parent_id)
            .collect();
        rows.sort_by_key(|r| r.id);
        serde_wasm_bindgen::to_value(&rows).unwrap_or(JsValue::UNDEFINED)
    }

    /// Returns `true` on success, `false` if the row doesn't exist.
    pub fn update_row(&mut self, id: u32, label: &str, fields: JsValue) -> bool {
        let fields: Vec<String> = serde_wasm_bindgen::from_value(fields).unwrap_or_default();
        matches!(self.handle(Request::UpdateRow { id, label: label.to_string(), fields }), Response::Updated)
    }

    pub fn delete_row(&mut self, id: u32) {
        self.handle(Request::DeleteRow(id));
    }
}

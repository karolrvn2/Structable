use crate::protocol::{Request, Response, Row};

pub trait Transport {
    fn send(&mut self, req: Request) -> Response;
}

pub struct Client<T: Transport> {
    transport: T,
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn get_row(&mut self, id: u32) -> Option<Row> {
        match self.transport.send(Request::GetRow(id)) {
            Response::Row(row) => row,
            _ => None,
        }
    }

    pub fn list_rows(&mut self, parent_id: Option<u32>) -> Vec<Row> {
        match self.transport.send(Request::ListRows { parent_id }) {
            Response::Rows(rows) => rows,
            _ => vec![],
        }
    }

    pub fn insert_row(&mut self, label: &str, parent_id: Option<u32>, fields: Vec<String>) -> u32 {
        match self.transport.send(Request::InsertRow {
            label: label.to_string(),
            parent_id,
            fields,
        }) {
            Response::Inserted(id) => id,
            _ => panic!("unexpected response to InsertRow"),
        }
    }

    pub fn update_row(&mut self, id: u32, label: &str, fields: Vec<String>) -> bool {
        matches!(
            self.transport.send(Request::UpdateRow { id, label: label.to_string(), fields }),
            Response::Updated
        )
    }

    pub fn delete_row(&mut self, id: u32) {
        self.transport.send(Request::DeleteRow(id));
    }
}

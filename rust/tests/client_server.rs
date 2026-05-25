use wasm_bindgen_test::*;

// Use `run_in_node_experimental` for headless CI; swap to `run_in_browser` for browser testing.
wasm_bindgen_test_configure!(run_in_node_experimental);

use structable_core::{
    client::{Client, Transport},
    protocol::{Request, Response},
    server::Server,
};

struct InMemoryTransport {
    server: Server,
}

impl InMemoryTransport {
    fn new() -> Self {
        Self { server: Server::new() }
    }
}

impl Transport for InMemoryTransport {
    fn send(&mut self, req: Request) -> Response {
        self.server.handle(req)
    }
}

fn make_client() -> Client<InMemoryTransport> {
    Client::new(InMemoryTransport::new())
}

#[wasm_bindgen_test]
fn insert_and_get() {
    let mut c = make_client();
    let id = c.insert_row("Alice", None, vec!["eng".into()]);
    let row = c.get_row(id).expect("row should exist");
    assert_eq!(row.label, "Alice");
    assert_eq!(row.fields, ["eng"]);
    assert_eq!(row.parent_id, None);
}

#[wasm_bindgen_test]
fn get_nonexistent_returns_none() {
    let mut c = make_client();
    assert!(c.get_row(999).is_none());
}

#[wasm_bindgen_test]
fn list_root_rows() {
    let mut c = make_client();
    c.insert_row("Alice", None, vec![]);
    c.insert_row("Bob", None, vec![]);
    c.insert_row("Child", Some(1), vec![]);
    let roots = c.list_rows(None);
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0].label, "Alice");
    assert_eq!(roots[1].label, "Bob");
}

#[wasm_bindgen_test]
fn list_children() {
    let mut c = make_client();
    let parent_id = c.insert_row("Parent", None, vec![]);
    c.insert_row("Child A", Some(parent_id), vec![]);
    c.insert_row("Child B", Some(parent_id), vec![]);
    let children = c.list_rows(Some(parent_id));
    assert_eq!(children.len(), 2);
    assert!(children.iter().all(|r| r.parent_id == Some(parent_id)));
}

#[wasm_bindgen_test]
fn list_empty_parent() {
    let mut c = make_client();
    assert!(c.list_rows(Some(42)).is_empty());
}

#[wasm_bindgen_test]
fn update_row() {
    let mut c = make_client();
    let id = c.insert_row("Original", None, vec!["old".into()]);
    assert!(c.update_row(id, "Updated", vec!["new".into()]));
    let row = c.get_row(id).expect("row should still exist");
    assert_eq!(row.label, "Updated");
    assert_eq!(row.fields, ["new"]);
}

#[wasm_bindgen_test]
fn update_nonexistent_returns_false() {
    let mut c = make_client();
    assert!(!c.update_row(999, "Ghost", vec![]));
}

#[wasm_bindgen_test]
fn delete_row() {
    let mut c = make_client();
    let id = c.insert_row("Temp", None, vec![]);
    assert!(c.get_row(id).is_some());
    c.delete_row(id);
    assert!(c.get_row(id).is_none());
}

#[wasm_bindgen_test]
fn delete_nonexistent_is_noop() {
    let mut c = make_client();
    c.delete_row(999); // should not panic
}

#[wasm_bindgen_test]
fn ids_are_unique_and_sequential() {
    let mut c = make_client();
    let a = c.insert_row("A", None, vec![]);
    let b = c.insert_row("B", None, vec![]);
    let cc = c.insert_row("C", None, vec![]);
    assert_ne!(a, b);
    assert_ne!(b, cc);
    assert_eq!(b, a + 1);
    assert_eq!(cc, a + 2);
}

#[wasm_bindgen_test]
fn deleted_id_not_in_list() {
    let mut c = make_client();
    let id = c.insert_row("Gone", None, vec![]);
    c.insert_row("Still here", None, vec![]);
    c.delete_row(id);
    let roots = c.list_rows(None);
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].label, "Still here");
}

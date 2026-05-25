enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

struct Cell {
    value: String,
}

struct Attribute {
    id: String,
}

// #[wasm_bindgen_test]
// fn fetch_snapshot() {
//     let peer = Peer::new();
//     let object = 
//     let attribute =
//     assert!(c.get_row(id).is_some());

//     ///
//     let c = make_client();
//     let id = c.insert_row("Temp", None, vec![]);
//     assert!(c.get_row(id).is_some());
//     c.delete_row(id);
//     assert!(c.get_row(id).is_none());
// }
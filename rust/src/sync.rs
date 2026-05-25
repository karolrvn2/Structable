use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::types::{Attribute, Object, Value};

// ── Op log types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpKind {
    CreateObject { object_id: u32, parent_id: Option<u32> },
    /// Last-write-wins per cell. `ts` on the Op is the tiebreaker.
    SetCell { object_id: u32, attribute_id: String, value: Value },
    DeleteObject { object_id: u32 },
    UpsertAttribute { attribute: Attribute },
    DeleteAttribute { attribute_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Op {
    pub client_id: String,
    /// Monotonically increasing per-client counter.
    pub seq: u64,
    /// Wall-clock milliseconds — used for LWW on SetCell.
    pub ts: u64,
    pub kind: OpKind,
}

impl Op {
    fn key(&self) -> (String, u64) {
        (self.client_id.clone(), self.seq)
    }
}

// ── Engine ────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct SyncEngine {
    objects: HashMap<u32, Object>,
    attributes: Vec<Attribute>,
    /// LWW stamps per cell: (object_id, attribute_id) → (ts, client_id).
    cell_stamps: HashMap<(u32, String), (u64, String)>,
    client_id: String,
    next_seq: u64,
    /// Applied locally; not yet ACKed by the server.
    pending: Vec<Op>,
    /// Deduplication set so remote merges are idempotent.
    applied: HashSet<(String, u64)>,
}

// ── Private Rust API ──────────────────────────────────────────────────────────

impl SyncEngine {
    fn apply_op(&mut self, op: &Op) {
        match &op.kind {
            OpKind::CreateObject { object_id, parent_id } => {
                self.objects.entry(*object_id).or_insert_with(|| Object {
                    id: *object_id,
                    parent_id: *parent_id,
                    cells: HashMap::new(),
                });
            }
            OpKind::SetCell { object_id, attribute_id, value } => {
                let stamp_key = (*object_id, attribute_id.clone());
                let wins = match self.cell_stamps.get(&stamp_key) {
                    None => true,
                    Some((stored_ts, stored_client)) =>
                        op.ts > *stored_ts
                        || (op.ts == *stored_ts && op.client_id > *stored_client),
                };
                if wins {
                    if let Some(obj) = self.objects.get_mut(object_id) {
                        obj.cells.insert(attribute_id.clone(), value.clone());
                        self.cell_stamps.insert(stamp_key, (op.ts, op.client_id.clone()));
                    }
                }
            }
            OpKind::DeleteObject { object_id } => {
                self.objects.remove(object_id);
            }
            OpKind::UpsertAttribute { attribute } => {
                match self.attributes.iter_mut().find(|a| a.id == attribute.id) {
                    Some(existing) => *existing = attribute.clone(),
                    None => self.attributes.push(attribute.clone()),
                }
            }
            OpKind::DeleteAttribute { attribute_id } => {
                self.attributes.retain(|a| a.id != *attribute_id);
                for obj in self.objects.values_mut() {
                    obj.cells.remove(attribute_id);
                }
            }
        }
        self.applied.insert(op.key());
    }

    fn emit(&mut self, ts: u64, kind: OpKind) -> Op {
        let seq = self.next_seq;
        self.next_seq += 1;
        let op = Op { client_id: self.client_id.clone(), seq, ts, kind };
        self.apply_op(&op.clone());
        self.pending.push(op.clone());
        op
    }

    fn next_object_id(&self) -> u32 {
        let client_hash = self.client_id
            .bytes()
            .fold(0u16, |acc, b| acc.wrapping_add(b as u16));
        ((client_hash as u32) << 16) | (self.next_seq as u32 & 0xFFFF)
    }
}

// ── Public WASM API ───────────────────────────────────────────────────────────

#[wasm_bindgen]
impl SyncEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(client_id: &str) -> SyncEngine {
        SyncEngine {
            objects: HashMap::new(),
            attributes: Vec::new(),
            cell_stamps: HashMap::new(),
            client_id: client_id.to_string(),
            next_seq: 0,
            pending: Vec::new(),
            applied: HashSet::new(),
        }
    }

    // ── Schema ────────────────────────────────────────────────────────────────

    /// `attr_json`: serialised `Attribute` object.
    pub fn upsert_attribute(&mut self, ts: u64, attr_json: &str) {
        if let Ok(attr) = serde_json::from_str::<Attribute>(attr_json) {
            self.emit(ts, OpKind::UpsertAttribute { attribute: attr });
        }
    }

    pub fn delete_attribute(&mut self, ts: u64, attribute_id: &str) {
        self.emit(ts, OpKind::DeleteAttribute { attribute_id: attribute_id.to_string() });
    }

    /// Returns JS array of `Attribute` objects.
    pub fn list_attributes(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.attributes).unwrap_or(JsValue::UNDEFINED)
    }

    // ── Objects ───────────────────────────────────────────────────────────────

    /// Returns the new object's id.
    pub fn create_object(&mut self, ts: u64, parent_id: Option<u32>) -> u32 {
        let object_id = self.next_object_id();
        self.emit(ts, OpKind::CreateObject { object_id, parent_id });
        object_id
    }

    /// `value_json`: a JSON-encoded `Value` (null / bool / number / string).
    pub fn set_cell(&mut self, ts: u64, object_id: u32, attribute_id: &str, value_json: &str) {
        let value: Value = serde_json::from_str(value_json).unwrap_or(Value::Null);
        self.emit(ts, OpKind::SetCell {
            object_id,
            attribute_id: attribute_id.to_string(),
            value,
        });
    }

    pub fn delete_object(&mut self, ts: u64, object_id: u32) {
        self.emit(ts, OpKind::DeleteObject { object_id });
    }

    /// Returns the `Object` as a JS object, or `undefined`.
    pub fn get_object(&self, id: u32) -> JsValue {
        match self.objects.get(&id) {
            Some(obj) => serde_wasm_bindgen::to_value(obj).unwrap_or(JsValue::UNDEFINED),
            None => JsValue::UNDEFINED,
        }
    }

    /// Returns a JS array of `Object`s whose `parent_id` matches.
    /// Pass `undefined` for root objects.
    pub fn list_objects(&self, parent_id: Option<u32>) -> JsValue {
        let mut objs: Vec<&Object> = self.objects.values()
            .filter(|o| o.parent_id == parent_id)
            .collect();
        objs.sort_by_key(|o| o.id);
        serde_wasm_bindgen::to_value(&objs).unwrap_or(JsValue::UNDEFINED)
    }

    // ── Sync protocol ─────────────────────────────────────────────────────────

    /// Returns a JSON string of pending (unsynced) ops to POST to the server.
    pub fn pending_ops(&self) -> String {
        serde_json::to_string(&self.pending).unwrap_or_else(|_| "[]".into())
    }

    /// Call after the server confirms it received all ops up to and including `seq`.
    pub fn ack_up_to(&mut self, seq: u32) {
        self.pending.retain(|op| op.seq > seq as u64);
    }

    /// Apply ops received from another client (or the server's canonical log).
    /// Idempotent: already-applied ops are skipped.
    pub fn merge_remote(&mut self, ops_json: &str) {
        if let Ok(ops) = serde_json::from_str::<Vec<Op>>(ops_json) {
            for op in ops {
                if !self.applied.contains(&op.key()) {
                    self.apply_op(&op);
                }
            }
        }
    }

    /// Restore pending ops persisted from a previous session (e.g. localStorage).
    /// Re-applies their effects and resumes the pending queue.
    pub fn restore_pending(&mut self, ops_json: &str) {
        if let Ok(ops) = serde_json::from_str::<Vec<Op>>(ops_json) {
            for op in &ops {
                if !self.applied.contains(&op.key()) {
                    self.apply_op(op);
                }
            }
            if let Some(max_seq) = ops.iter()
                .filter(|op| op.client_id == self.client_id)
                .map(|op| op.seq)
                .max()
            {
                self.next_seq = max_seq + 1;
            }
            self.pending = ops;
        }
    }
}

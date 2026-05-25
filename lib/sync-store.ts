import type { SyncEngine } from '../pkg/structable_core';

// Mirrors the Rust Value enum (serde untagged → plain JSON types).
export type Value = string | number | boolean | null;

export type SObject = {
  id: number;
  parent_id: number | null;
  cells: Record<string, Value>;
};

export type Attribute = {
  id: string;
  label: string;
  kind: 'text' | 'number' | 'checkbox';
};

type Listener = () => void;

const CLIENT_ID_KEY = 'structable:client_id';
const PENDING_KEY   = 'structable:pending_ops';

function getClientId(): string {
  let id = localStorage.getItem(CLIENT_ID_KEY);
  if (!id) {
    id = crypto.randomUUID();
    localStorage.setItem(CLIENT_ID_KEY, id);
  }
  return id;
}

// ── SyncStore ─────────────────────────────────────────────────────────────────

export class SyncStore {
  private engine: SyncEngine | null = null;
  private listeners = new Set<Listener>();
  private syncTimer: ReturnType<typeof setTimeout> | null = null;
  private initPromise: Promise<void> | null = null;
  readonly syncEndpoint: string;

  constructor(syncEndpoint = '/api/sync') {
    this.syncEndpoint = syncEndpoint;
  }

  /** Safe to call multiple times — returns the same promise. */
  init(): Promise<void> {
    if (!this.initPromise) this.initPromise = this._init();
    return this.initPromise;
  }

  private async _init(): Promise<void> {
    const { SyncEngine: WasmSyncEngine } = await import('../pkg/structable_core');
    this.engine = new WasmSyncEngine(getClientId());

    const stored = localStorage.getItem(PENDING_KEY);
    if (stored && stored !== '[]') {
      this.engine.restore_pending(stored);
    }

    window.addEventListener('online', () => this.sync());
    if (navigator.onLine) this.scheduleSync();
  }

  // ── Schema ──────────────────────────────────────────────────────────────────

  upsertAttribute(attr: Attribute): void {
    this.engine!.upsert_attribute(BigInt(Date.now()), JSON.stringify(attr));
    this.flush();
  }

  deleteAttribute(attributeId: string): void {
    this.engine!.delete_attribute(BigInt(Date.now()), attributeId);
    this.flush();
  }

  listAttributes(): Attribute[] {
    return (this.engine?.list_attributes() ?? []) as Attribute[];
  }

  // ── Objects ─────────────────────────────────────────────────────────────────

  createObject(parentId?: number): number {
    return this.engine!.create_object(BigInt(Date.now()), parentId);
  }

  setCell(objectId: number, attributeId: string, value: Value): void {
    this.engine!.set_cell(BigInt(Date.now()), objectId, attributeId, JSON.stringify(value));
    this.flush();
  }

  deleteObject(objectId: number): void {
    this.engine!.delete_object(BigInt(Date.now()), objectId);
    this.flush();
  }

  listObjects(parentId?: number): SObject[] {
    return (this.engine?.list_objects(parentId) ?? []) as SObject[];
  }

  getObject(id: number): SObject | undefined {
    const v = this.engine?.get_object(id);
    return v == null ? undefined : (v as SObject);
  }

  // ── Sync ────────────────────────────────────────────────────────────────────

  get hasPending(): boolean {
    return this.engine?.pending_ops() !== '[]';
  }

  get isReady(): boolean {
    return this.engine !== null;
  }

  async sync(): Promise<void> {
    if (!this.engine || !navigator.onLine) return;
    const pending = this.engine.pending_ops();
    if (pending === '[]') return;

    try {
      const res = await fetch(this.syncEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ops: JSON.parse(pending) }),
      });
      if (!res.ok) return;

      // Server contract:
      //   { acked_seq: number, new_ops: Op[] }
      // acked_seq: highest seq (for this client) the server has durably stored.
      // new_ops:   ops from other clients not yet seen by this client.
      const { acked_seq, new_ops } = await res.json() as {
        acked_seq: number;
        new_ops: unknown[];
      };

      this.engine.ack_up_to(acked_seq);
      if (new_ops.length > 0) {
        this.engine.merge_remote(JSON.stringify(new_ops));
      }
      this.flush();
    } catch {
      // Offline or server error — will retry on next flush.
    }
  }

  // ── Subscription ────────────────────────────────────────────────────────────

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  // ── Internal ────────────────────────────────────────────────────────────────

  private flush(): void {
    localStorage.setItem(PENDING_KEY, this.engine!.pending_ops());
    this.notify();
    this.scheduleSync();
  }

  private notify(): void {
    this.listeners.forEach(fn => fn());
  }

  private scheduleSync(ms = 2000): void {
    if (this.syncTimer) clearTimeout(this.syncTimer);
    this.syncTimer = setTimeout(() => this.sync(), ms);
  }
}

export const store = new SyncStore();

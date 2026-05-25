'use client';

import { useEffect, useReducer } from 'react';
import { store, type Attribute, type SObject, type Value } from '../lib/sync-store';

export function useStore(parentId?: number) {
  // Increment a counter whenever the store notifies us of a change.
  const [, rerender] = useReducer((n: number) => n + 1, 0);

  useEffect(() => {
    // Start WASM init and re-render when it's done.
    store.init().then(rerender);
    // Subscribe to all subsequent mutations.
    return store.subscribe(rerender);
  // parentId intentionally omitted — listObjects is called at read time.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    ready: store.isReady,
    hasPending: store.hasPending,

    // Reads (cheap — just a HashMap iteration in WASM)
    objects:    store.isReady ? store.listObjects(parentId) : ([] as SObject[]),
    attributes: store.isReady ? store.listAttributes()      : ([] as Attribute[]),

    // Schema mutations
    upsertAttribute: (attr: Attribute)   => store.upsertAttribute(attr),
    deleteAttribute: (id: string)        => store.deleteAttribute(id),

    // Object mutations
    createObject: (pid?: number)                              => store.createObject(pid),
    setCell:      (oid: number, aid: string, val: Value)      => store.setCell(oid, aid, val),
    deleteObject: (id: number)                                => store.deleteObject(id),

    // Manual sync trigger (normally auto on mutation + online event)
    sync: () => store.sync(),
  };
}

'use client';

import React, { useEffect, useMemo, useRef, useState } from 'react';

type Column = {
  id: string;
  label: string;
  width?: number;
  minWidth?: number;
  subColumns?: Column[];
};

type HeaderCell = {
  column: Column;
  colSpan: number;
  rowSpan: number;
  depth: number;
};

type HeaderTraversalResult = {
  colSpan: number;
  rowSpan: number;
  visible: boolean;
};

const columns: Column[] = [
  {
    id: 'personal',
    label: 'Personal',
    subColumns: [
      {
        id: 'identity',
        label: 'Identity',
        subColumns: [
          { id: 'firstName', label: 'First Name', width: 170, minWidth: 120 },
          { id: 'lastName', label: 'Last Name', width: 170, minWidth: 120 },
        ],
      },
      {
        id: 'contact',
        label: 'Contact',
        subColumns: [
          { id: 'email', label: 'Email', width: 260, minWidth: 160 },
          { id: 'phone', label: 'Phone', width: 150, minWidth: 120 },
        ],
      },
    ],
  },
  {
    id: 'work',
    label: 'Work',
    subColumns: [
      {
        id: 'position',
        label: 'Position',
        width: 200,
        minWidth: 140,
      },
      {
        id: 'department',
        label: 'Department',
        width: 180,
        minWidth: 140,
      },
    ],
  },
  {
    id: 'performance',
    label: 'Performance',
    subColumns: [
      {
        id: 'scoreGroup',
        label: 'Scores',
        subColumns: [
          { id: 'score', label: 'Score', width: 110, minWidth: 90 },
          { id: 'rank', label: 'Rank', width: 100, minWidth: 90 },
        ],
      },
      {
        id: 'trend',
        label: 'Trend',
        width: 120,
        minWidth: 90,
      },
    ],
  },
];

const rows = [
  {
    firstName: 'Avery',
    lastName: 'Taylor',
    email: 'avery.taylor@example.com',
    phone: '+1 (415) 555-0123',
    position: 'Product Lead',
    department: 'Design',
    score: '93',
    rank: '1',
    trend: '↗',
  },
  {
    firstName: 'Jordan',
    lastName: 'Reese',
    email: 'jordan.reese@example.com',
    phone: '+1 (212) 555-0145',
    position: 'Lead Engineer',
    department: 'Platform',
    score: '88',
    rank: '3',
    trend: '→',
  },
  {
    firstName: 'Mina',
    lastName: 'Cho',
    email: 'mina.cho@example.com',
    phone: '+1 (646) 555-0192',
    position: 'Analytics Manager',
    department: 'Data',
    score: '90',
    rank: '2',
    trend: '↗',
  },
  {
    firstName: 'Noah',
    lastName: 'Jackson',
    email: 'noah.jackson@example.com',
    phone: '+1 (323) 555-0176',
    position: 'Sales Director',
    department: 'Revenue',
    score: '81',
    rank: '5',
    trend: '↘',
  },
  {
    firstName: 'Layla',
    lastName: 'Patel',
    email: 'layla.patel@example.com',
    phone: '+1 (503) 555-0114',
    position: 'Finance Analyst',
    department: 'Operations',
    score: '85',
    rank: '4',
    trend: '→',
  },
];

function getLeafColumns(columnList: Column[]): Column[] {
  return columnList.flatMap((column) =>
    column.subColumns ? getLeafColumns(column.subColumns) : [column],
  );
}

function getColumnDepth(columnList: Column[]): number {
  return columnList.reduce((depth, column) => {
    if (!column.subColumns) {
      return Math.max(depth, 1);
    }
    return Math.max(depth, 1 + getColumnDepth(column.subColumns));
  }, 0);
}

function buildHeaderRows(columnList: Column[], maxDepth: number, hiddenSet: Set<string>) {
  const rows: HeaderCell[][] = Array.from({ length: maxDepth }, () => [] as HeaderCell[]);

  function traverse(column: Column, depth: number): HeaderTraversalResult {
    const isLeaf = !column.subColumns || column.subColumns.length === 0;

    if (isLeaf) {
      if (hiddenSet.has(column.id)) {
        return { colSpan: 0, rowSpan: 0, visible: false };
      }
      const rowSpan = maxDepth - depth + 1;
      rows[depth - 1].push({ column, colSpan: 1, rowSpan, depth });
      return { colSpan: 1, rowSpan, visible: true };
    }

    const childResults = column.subColumns!.map((child) => traverse(child, depth + 1));
    const colSpan = childResults.reduce((sum, result) => sum + (result.visible ? result.colSpan : 0), 0);

    if (colSpan === 0) {
      return { colSpan: 0, rowSpan: 0, visible: false };
    }

    rows[depth - 1].push({ column, colSpan, rowSpan: 1, depth });
    return { colSpan, rowSpan: 1, visible: true };
  }

  columnList.forEach((column) => traverse(column, 1));
  return rows.map((row) => row.filter((cell) => cell.colSpan > 0));
}

export default function TreeTable() {
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [hiddenColumns, setHiddenColumns] = useState<string[]>([]);
  const [openMenuColumn, setOpenMenuColumn] = useState<string | null>(null);
  const [columnWidths, setColumnWidths] = useState<Record<string, number>>(() => {
    const widths: Record<string, number> = {};
    getLeafColumns(columns).forEach((column) => {
      widths[column.id] = column.width ?? 130;
    });
    return widths;
  });

  const hiddenSet = useMemo(() => new Set(hiddenColumns), [hiddenColumns]);
  const selectedSet = useMemo(() => new Set(selectedKeys), [selectedKeys]);
  const lastSelectedRef = useRef<string | null>(null);
  const resizingRef = useRef<{
    id: string;
    startX: number;
    startWidth: number;
  } | null>(null);

  const allLeafColumns = useMemo<Column[]>(() => getLeafColumns(columns), []);
  const visibleLeafColumns = useMemo(
    () => allLeafColumns.filter((column) => !hiddenSet.has(column.id)),
    [allLeafColumns, hiddenSet],
  );
  const headerRows = useMemo<HeaderCell[][]>(
    () => buildHeaderRows(columns, getColumnDepth(columns), hiddenSet),
    [hiddenSet],
  );

  const hideColumn = (columnId: string) => {
    setHiddenColumns((prev) => (prev.includes(columnId) ? prev : [...prev, columnId]));
    setOpenMenuColumn(null);
  };

  const showAllColumns = () => {
    setHiddenColumns([]);
    setOpenMenuColumn(null);
  };

  useEffect(() => {
    function handlePointerMove(event: PointerEvent) {
      if (!resizingRef.current) {
        return;
      }
      const { id, startX, startWidth } = resizingRef.current;
      const delta = event.clientX - startX;
      setColumnWidths((prev) => {
        const minWidth = allLeafColumns.find((col) => col.id === id)?.minWidth ?? 80;
        const next = Math.max(startWidth + delta, minWidth);
        return { ...prev, [id]: next };
      });
    }

    function handlePointerUp() {
      resizingRef.current = null;
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    }

    if (resizingRef.current) {
      window.addEventListener('pointermove', handlePointerMove);
      window.addEventListener('pointerup', handlePointerUp);
      return () => {
        window.removeEventListener('pointermove', handlePointerMove);
        window.removeEventListener('pointerup', handlePointerUp);
      };
    }
  }, [allLeafColumns]);

  useEffect(() => {
    if (!openMenuColumn) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as HTMLElement;
      if (!target.closest('.column-menu') && !target.closest('.column-action-button')) {
        setOpenMenuColumn(null);
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [openMenuColumn]);

  const handleResizerPointerDown = (
    event: React.PointerEvent<HTMLDivElement>,
    columnId: string,
  ) => {
    event.preventDefault();
    resizingRef.current = {
      id: columnId,
      startX: event.clientX,
      startWidth: columnWidths[columnId],
    };
  };

  const toggleSelection = (key: string, event: React.MouseEvent<HTMLTableCellElement>) => {
    setSelectedKeys((prev) => {
      const isSelected = prev.includes(key);
      const shouldKeepMultiple = event.metaKey || event.ctrlKey;
      const next = shouldKeepMultiple ? [...prev] : [];

      if (shouldKeepMultiple) {
        if (isSelected) {
          return next.filter((item) => item !== key);
        }
        return [...next, key];
      }

      return isSelected ? [] : [key];
    });
    lastSelectedRef.current = key;
  };

  return (
    <div className="tree-table-wrap">
      <table className="tree-table">
        <colgroup>
          {visibleLeafColumns.map((column) => (
            <col
              key={column.id}
              style={{ width: `${columnWidths[column.id]}px`, minWidth: `${column.minWidth ?? 80}px` }}
            />
          ))}
        </colgroup>
        <thead>
          {headerRows.map((row, rowIndex) => (
            <tr key={rowIndex}>
              {row.map(({ column, colSpan, rowSpan, depth }) => {
                const headerKey = `header:${column.id}:${depth}`;
                const isHeaderSelected = selectedSet.has(headerKey);
                const isLeaf = !column.subColumns || column.subColumns.length === 0;
                const leafColumn = isLeaf ? visibleLeafColumns.find((leaf) => leaf.id === column.id) : undefined;

                return (
                  <th
                    key={headerKey}
                    colSpan={colSpan}
                    rowSpan={rowSpan}
                    className={isHeaderSelected ? 'header-selected' : undefined}
                    onClick={(event) => toggleSelection(headerKey, event)}
                  >
                    <div className="column-group">
                      {column.label}
                      {isLeaf && leafColumn ? (
                        <span className="header-actions">
                          <button
                            type="button"
                            className="column-action-button"
                            onClick={(event) => {
                              event.stopPropagation();
                              setOpenMenuColumn((prev) => (prev === column.id ? null : column.id));
                            }}
                            aria-label={`Column actions for ${column.label}`}
                          >
                            ☰
                          </button>
                          {openMenuColumn === column.id ? (
                            <div className="column-menu">
                              <button
                                type="button"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  hideColumn(column.id);
                                }}
                              >
                                Hide column
                              </button>
                            </div>
                          ) : null}
                          <div
                            className="resizer"
                            onPointerDown={(event) => handleResizerPointerDown(event, column.id)}
                          />
                        </span>
                      ) : null}
                    </div>
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody>
          {rows.map((row, rowIndex) => (
            <tr key={rowIndex}>
              {visibleLeafColumns.map((column) => {
                const bodyKey = `cell:${rowIndex}:${column.id}`;
                const isSelected = selectedSet.has(bodyKey);
                return (
                  <td
                    key={bodyKey}
                    className={isSelected ? 'cell-selected' : undefined}
                    onClick={(event) => toggleSelection(bodyKey, event)}
                  >
                    {row[column.id as keyof typeof row] ?? '-'}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="table-note">
        Tip: Ctrl/Cmd + click to select multiple cells, including header groups.
        {hiddenColumns.length > 0 ? (
          <button type="button" className="show-all-button" onClick={showAllColumns}>
            Show all hidden columns ({hiddenColumns.length})
          </button>
        ) : null}
      </div>
    </div>
  );
}

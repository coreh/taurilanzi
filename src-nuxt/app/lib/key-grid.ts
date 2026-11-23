export type Direction = 'up' | 'down' | 'left' | 'right'

/** The D200's keys as they sit on the dock: the wide key (13) takes the last two columns. */
const ROWS: number[][] = [
  [0, 1, 2, 3, 4],
  [5, 6, 7, 8, 9],
  [10, 11, 12, 13, 13],
]

export const ARROW_DIRECTIONS: Record<string, Direction> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
}

function place(index: number) {
  const row = ROWS.findIndex(r => r.includes(index))
  if (row === -1) {
    return null
  }
  const columns = ROWS[row]!.flatMap((key, column) => (key === index ? [column] : []))
  return { row, first: columns[0]!, last: columns.at(-1)! }
}

/**
 * The key next to `index` in that direction, or null at the edge of the dock. `column` is the
 * column a vertical move came through, so going down into the wide key and back up returns to
 * the key it started from instead of always to the one above its left half.
 */
export function neighbourKey(index: number, direction: Direction, column?: number): { index: number, column: number } | null {
  const at = place(index)
  if (!at) {
    return null
  }
  if (direction === 'left' || direction === 'right') {
    const next = direction === 'left' ? at.first - 1 : at.last + 1
    const key = ROWS[at.row]![next]
    return key === undefined ? null : { index: key, column: next }
  }
  const through = column !== undefined && column >= at.first && column <= at.last ? column : at.first
  const key = ROWS[at.row + (direction === 'up' ? -1 : 1)]?.[through]
  return key === undefined ? null : { index: key, column: through }
}

export type DiffCellKind = 'context' | 'add' | 'del' | 'empty'

export interface DiffCell {
  line: number | null
  text: string
  kind: DiffCellKind
}

export interface SplitDiffRow {
  kind: 'hunk' | 'context' | 'change' | 'meta'
  text?: string
  left?: DiffCell
  right?: DiffCell
}

function emptyCell(): DiffCell {
  return { line: null, text: '', kind: 'empty' }
}

export function parseSplitDiff(patch: string): SplitDiffRow[] {
  if (!patch.trim()) return []
  const rows: SplitDiffRow[] = []
  let oldLine = 0
  let newLine = 0
  let deletions: DiffCell[] = []
  let additions: DiffCell[] = []

  const flush = (): void => {
    while (deletions.length > 0 || additions.length > 0) {
      rows.push({
        kind: 'change',
        left: deletions.shift() ?? emptyCell(),
        right: additions.shift() ?? emptyCell(),
      })
    }
  }

  for (const line of patch.split('\n')) {
    if (line.startsWith('@@')) {
      flush()
      const match = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line)
      if (match) {
        oldLine = Number(match[1])
        newLine = Number(match[2])
      }
      rows.push({ kind: 'hunk', text: line })
      continue
    }
    if (line.startsWith('---') || line.startsWith('+++')) {
      continue
    }
    if (line.startsWith('\\')) {
      flush()
      rows.push({ kind: 'meta', text: line })
      continue
    }
    if (line.length === 0) continue
    const prefix = line[0]
    const text = line.slice(1)
    if (prefix === '-') {
      deletions.push({ line: oldLine, text, kind: 'del' })
      oldLine += 1
      continue
    }
    if (prefix === '+') {
      additions.push({ line: newLine, text, kind: 'add' })
      newLine += 1
      continue
    }
    if (prefix === ' ') {
      flush()
      rows.push({
        kind: 'context',
        left: { line: oldLine, text, kind: 'context' },
        right: { line: newLine, text, kind: 'context' },
      })
      oldLine += 1
      newLine += 1
      continue
    }
    flush()
    rows.push({ kind: 'meta', text: line })
  }
  flush()
  return rows
}

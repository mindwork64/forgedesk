import { describe, expect, it } from 'vitest'

import { parseSplitDiff } from './splitDiff'

describe('split diff parser', () => {
  const patch = [
    '@@ -1,4 +1,5 @@',
    ' const a = 1;',
    '-const b = 2;',
    '+const b = 3;',
    '+const c = 4;',
    ' const d = 5;',
  ].join('\n')

  it('pairs deletions and additions with line numbers', () => {
    const rows = parseSplitDiff(patch)
    expect(rows[0]).toMatchObject({ kind: 'hunk' })
    expect(rows[1]).toMatchObject({
      kind: 'context',
      left: { line: 1, text: 'const a = 1;' },
      right: { line: 1, text: 'const a = 1;' },
    })
    expect(rows[2]).toMatchObject({
      kind: 'change',
      left: { line: 2, text: 'const b = 2;', kind: 'del' },
      right: { line: 2, text: 'const b = 3;', kind: 'add' },
    })
    expect(rows[3]).toMatchObject({
      kind: 'change',
      left: { line: null, kind: 'empty' },
      right: { line: 3, text: 'const c = 4;', kind: 'add' },
    })
    expect(rows[4]).toMatchObject({
      kind: 'context',
      left: { line: 3 },
      right: { line: 4 },
    })
  })

  it('handles pure additions', () => {
    const rows = parseSplitDiff('@@ -0,0 +1,2 @@\n+one\n+two')
    expect(rows.slice(1)).toEqual([
      {
        kind: 'change',
        left: { line: null, text: '', kind: 'empty' },
        right: { line: 1, text: 'one', kind: 'add' },
      },
      {
        kind: 'change',
        left: { line: null, text: '', kind: 'empty' },
        right: { line: 2, text: 'two', kind: 'add' },
      },
    ])
  })

  it('keeps no-newline metadata', () => {
    const rows = parseSplitDiff('@@ -1 +1 @@\n-a\n\\ No newline at end of file\n+b')
    expect(rows.some((row) => row.kind === 'meta')).toBe(true)
  })

  it('returns no rows for an empty patch', () => {
    expect(parseSplitDiff('')).toEqual([])
  })
})

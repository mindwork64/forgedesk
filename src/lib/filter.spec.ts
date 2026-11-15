import { describe, expect, it } from 'vitest'

import {
  countFilterConditions,
  defaultCondition,
  filterChipLabel,
  isFilterComplete,
  pruneFilter,
  type FilterMetadata,
} from './filter'
import type { FilterGroup, Label, StatusSummary } from '@/types/ipc'

const status: StatusSummary = {
  id: 'st-todo',
  name: 'Todo',
  category: 'unstarted',
  color: 'info',
  position: 1,
  systemKey: 'todo',
}

const label: Label = {
  id: 'label-capture',
  name: 'capture',
  color: 'neutral',
}

const metadata: FilterMetadata = {
  statuses: [status],
  labels: [label],
  projects: [],
  issues: [],
}

const t = (key: string, params?: Record<string, unknown>): string => {
  const values: Record<string, string> = {
    'filters.fields.status': 'Status',
    'filters.fields.priority': 'Priority',
    'filters.operators.eq': 'is',
    'filters.operators.gte': 'is on or after',
    'priority.high': 'High',
  }
  if (key === 'filters.groupChip') {
    return `Group (${String(params?.count)})`
  }
  return values[key] ?? key
}
const te = (key: string): boolean => key in {
  'filters.fields.status': true,
  'filters.fields.priority': true,
  'filters.operators.eq': true,
  'filters.operators.gte': true,
  'priority.high': true,
}

describe('filter helpers', () => {
  it('creates complete defaults for common fields', () => {
    expect(defaultCondition('status', metadata)).toMatchObject({
      field: 'status',
      operator: 'eq',
      value: 'st-todo',
    })
    expect(defaultCondition('priority', metadata).value).toBe('medium')
    expect(defaultCondition('label', metadata).value).toBe('label-capture')
    expect(defaultCondition('title', metadata).operator).toBe('contains')
  })

  it('validates values and prunes empty groups', () => {
    const empty = defaultCondition('title', metadata)
    expect(isFilterComplete(empty)).toBe(false)
    empty.value = 'capture'
    expect(isFilterComplete(empty)).toBe(true)

    const emptyGroup: FilterGroup = {
      kind: 'group',
      op: 'or',
      conditions: [{ kind: 'group', op: 'and', conditions: [] }],
    }
    expect(pruneFilter(emptyGroup)).toBeNull()

    const group: FilterGroup = {
      kind: 'group',
      op: 'and',
      conditions: [
        defaultCondition('status', metadata),
        { kind: 'group', op: 'or', conditions: [empty] },
      ],
    }
    expect(countFilterConditions(group)).toBe(2)
    expect(pruneFilter(group)?.kind).toBe('group')
  })

  it('renders a human-readable chip', () => {
    expect(
      filterChipLabel(defaultCondition('status', metadata), metadata, t, te),
    ).toBe('Status is Todo')
  })
})

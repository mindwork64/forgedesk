import type {
  FilterCondition,
  FilterField,
  FilterGroup,
  FilterNode,
  FilterOperator,
  IssueSummary,
  Label,
  Priority,
  ProjectRef,
  StatusSummary,
} from '@/types/ipc'

type Translate = (key: string, params?: Record<string, unknown>) => string
type HasKey = (key: string) => boolean

export interface FilterMetadata {
  statuses: StatusSummary[]
  labels: Label[]
  projects: ProjectRef[]
  issues: IssueSummary[]
}

export const FILTER_FIELDS: FilterField[] = [
  'status',
  'statusCategory',
  'priority',
  'type',
  'project',
  'label',
  'estimate',
  'startDate',
  'dueDate',
  'createdAt',
  'updatedAt',
  'title',
  'parent',
  'archived',
  'source',
]

export function operatorsFor(field: FilterField): FilterOperator[] {
  if (field === 'title') {
    return ['contains', 'isEmpty', 'isNotEmpty']
  }
  if (field === 'archived') {
    return ['eq', 'neq', 'isEmpty', 'isNotEmpty']
  }
  if (
    field === 'estimate' ||
    field === 'startDate' ||
    field === 'dueDate' ||
    field === 'createdAt' ||
    field === 'updatedAt'
  ) {
    return ['eq', 'neq', 'gt', 'gte', 'lt', 'lte', 'isEmpty', 'isNotEmpty']
  }
  return ['eq', 'neq', 'isEmpty', 'isNotEmpty']
}

export function fieldLabel(
  field: FilterField,
  t: Translate,
  te: HasKey,
): string {
  const key = `filters.fields.${field}`
  return te(key) ? t(key) : field
}

export function operatorLabel(operator: FilterOperator, t: Translate): string {
  return t(`filters.operators.${operator}`)
}

export function countFilterConditions(node: FilterNode | null): number {
  if (!node) {
    return 0
  }
  if (node.kind === 'condition') {
    return 1
  }
  return node.conditions.reduce(
    (total, child) => total + countFilterConditions(child),
    0,
  )
}

export function isConditionComplete(
  condition: FilterCondition,
): boolean {
  if (condition.operator === 'isEmpty' || condition.operator === 'isNotEmpty') {
    return true
  }
  const value = condition.value
  if (value === null || value === undefined) {
    return false
  }
  if (typeof value === 'string') {
    return value.trim().length > 0
  }
  if (typeof value === 'number') {
    return Number.isFinite(value)
  }
  return typeof value === 'boolean'
}

export function isFilterComplete(node: FilterNode | null): boolean {
  if (!node) {
    return true
  }
  if (node.kind === 'condition') {
    return isConditionComplete(node)
  }
  if (node.conditions.length === 0) {
    return false
  }
  return node.conditions.every(isFilterComplete)
}

export function pruneFilter(node: FilterGroup): FilterNode | null {
  const conditions = node.conditions
    .map((child) => {
      if (child.kind === 'group') {
        const pruned = pruneFilter(child)
        return pruned
      }
      return child
    })
    .filter((child): child is FilterNode => child !== null)
    .filter((child) => child.kind !== 'group' || child.conditions.length > 0)

  if (conditions.length === 0) {
    return null
  }
  return { ...node, conditions }
}

export function valueLabel(
  condition: FilterCondition,
  metadata: FilterMetadata,
  t: Translate,
  te: HasKey,
): string | null {
  if (condition.operator === 'isEmpty' || condition.operator === 'isNotEmpty') {
    return null
  }
  const value = condition.value
  if (value === null || value === undefined || value === '') {
    return null
  }
  switch (condition.field) {
    case 'status':
      return metadata.statuses.find((status) => status.id === value)?.name ?? String(value)
    case 'statusCategory': {
      const key = `filters.categories.${String(value)}`
      return te(key) ? t(key) : String(value)
    }
    case 'priority': {
      const key = `priority.${String(value)}`
      return te(key) ? t(key) : String(value)
    }
    case 'type': {
      const key = `issueType.${String(value)}`
      return te(key) ? t(key) : String(value)
    }
    case 'project':
      return metadata.projects.find((project) => project.id === value)?.name ?? String(value)
    case 'label':
      return metadata.labels.find((label) => label.id === value)?.name ?? String(value)
    case 'parent':
      return metadata.issues.find((issue) => issue.id === value)?.key ?? String(value)
    case 'source': {
      const key = `filters.sources.${String(value)}`
      return te(key) ? t(key) : String(value)
    }
    case 'archived':
      return value === true ? t('filters.archivedTrue') : t('filters.archivedFalse')
    default:
      return String(value)
  }
}

export function filterChipLabel(
  condition: FilterCondition,
  metadata: FilterMetadata,
  t: Translate,
  te: HasKey,
): string {
  const field = fieldLabel(condition.field, t, te)
  const operator = operatorLabel(condition.operator, t)
  const value = valueLabel(condition, metadata, t, te)
  return value ? `${field} ${operator} ${value}` : `${field} ${operator}`
}

export function defaultCondition(
  field: FilterField,
  metadata: FilterMetadata,
): FilterCondition {
  const base: FilterCondition = {
    kind: 'condition',
    field,
    operator: operatorsFor(field)[0] ?? 'eq',
    value: undefined,
  }
  switch (field) {
    case 'status':
      base.value = metadata.statuses[0]?.id ?? ''
      break
    case 'statusCategory':
      base.value = 'started'
      break
    case 'priority':
      base.value = 'medium' satisfies Priority
      break
    case 'type':
      base.value = 'task'
      break
    case 'project':
      base.value = metadata.projects[0]?.id ?? ''
      break
    case 'label':
      base.value = metadata.labels[0]?.id ?? ''
      break
    case 'parent':
      base.value = metadata.issues[0]?.id ?? ''
      break
    case 'source':
      base.value = 'local'
      break
    case 'archived':
      base.value = true
      break
    case 'estimate':
      base.operator = 'gte'
      base.value = 0
      break
    case 'startDate':
    case 'dueDate':
    case 'createdAt':
    case 'updatedAt':
      base.operator = 'gte'
      base.value = ''
      break
    case 'title':
      base.operator = 'contains'
      base.value = ''
      break
  }
  return base
}

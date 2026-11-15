import type {
  ActivityEvent,
  IssueType,
  Priority,
  RelationKind,
  StatusColor,
  StatusSummary,
} from '@/types/ipc'

type Translate = (key: string, params?: Record<string, unknown>) => string
type HasKey = (key: string) => boolean

export function statusLabel(
  status: StatusSummary,
  t: Translate,
  te: HasKey,
): string {
  if (status.systemKey) {
    const key = `statuses.default.${status.systemKey}`
    if (te(key)) {
      return t(key)
    }
  }
  return status.name
}

export function priorityLabel(priority: Priority, t: Translate): string {
  return t(`priority.${priority}`)
}

export function issueTypeLabel(type: IssueType, t: Translate): string {
  return t(`issueType.${type}`)
}

export function relationKindLabel(kind: RelationKind, t: Translate): string {
  return t(`relation.${kind}`)
}

export function statusColorVar(color: StatusColor): string {
  return color === 'neutral' ? 'var(--fd-text-muted)' : `var(--fd-${color})`
}

function stringField(data: unknown, field: string): string | null {
  if (typeof data !== 'object' || data === null) {
    return null
  }
  const value = (data as Record<string, unknown>)[field]
  return typeof value === 'string' ? value : null
}

function stringArrayField(data: unknown, field: string): string[] {
  if (typeof data !== 'object' || data === null) {
    return []
  }
  const value = (data as Record<string, unknown>)[field]
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : []
}

export function activityText(
  event: ActivityEvent,
  t: Translate,
  te: HasKey,
): string {
  switch (event.kind) {
    case 'issue_created':
      return t('issues.activityCreated')
    case 'issue_updated': {
      const fields = stringArrayField(event.data, 'fields')
        .map((field) => {
          const key = `issues.fields.${field}`
          return te(key) ? t(key) : field
        })
        .join(', ')
      return fields
        ? t('issues.activityUpdated', { fields })
        : t('issues.activityUpdated', { fields: '—' })
    }
    case 'status_changed': {
      const from = stringField(event.data, 'from') ?? '—'
      const to = stringField(event.data, 'to') ?? '—'
      return t('issues.activityStatus', { from, to })
    }
    case 'labels_changed':
      return t('issues.activityLabels')
    case 'comment_added':
      return t('issues.activityComment')
    case 'relation_added':
    case 'relation_removed':
      return t('issues.activityRelation')
    case 'archived':
      return t('issues.activityArchived')
    case 'unarchived':
      return t('issues.activityUnarchived')
    case 'duplicated':
      return t('issues.activityDuplicated')
    default:
      return event.kind
  }
}

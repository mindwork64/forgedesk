type Translate = (key: string, params?: Record<string, unknown>) => string
import type {
  Cycle,
  CycleStatus,
  MilestoneStatus,
  PlanningStatus,
  Project,
} from '@/types/ipc'

export const PLANNING_STATUSES: PlanningStatus[] = [
  'planned',
  'active',
  'paused',
  'completed',
  'canceled',
]

export const MILESTONE_STATUSES: MilestoneStatus[] = [
  'planned',
  'active',
  'completed',
  'canceled',
]

export function planningStatusLabel(status: PlanningStatus, t: Translate): string {
  return t(`planning.status.${status}`)
}

export function milestoneStatusLabel(
  status: MilestoneStatus,
  t: Translate,
): string {
  return t(`planning.milestoneStatus.${status}`)
}

export function cycleStatusLabel(status: CycleStatus, t: Translate): string {
  return t(`cycles.status.${status}`)
}

export function percentLabel(percent: number): string {
  return `${Math.round(percent)}%`
}

export function formatShortDate(value: string | null, locale: string): string {
  if (!value) {
    return '—'
  }
  const date = new Date(`${value}T12:00:00`)
  return new Intl.DateTimeFormat(locale, {
    month: 'short',
    day: 'numeric',
  }).format(date)
}

export function formatDateRange(
  start: string | null,
  end: string | null,
  locale: string,
): string {
  if (!start && !end) {
    return ''
  }
  return `${formatShortDate(start, locale)} – ${formatShortDate(end, locale)}`
}

export function cycleRange(cycle: Cycle, locale: string): string {
  return formatDateRange(cycle.startDate, cycle.endDate, locale)
}

export function projectRange(project: Project, locale: string): string {
  return formatDateRange(project.startDate, project.targetDate, locale)
}

export function daysBetween(start: string, end: string): number {
  const startDate = new Date(`${start}T00:00:00Z`)
  const endDate = new Date(`${end}T00:00:00Z`)
  return Math.round((endDate.getTime() - startDate.getTime()) / 86_400_000)
}

export function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

export function dateOffsetIso(days: number): string {
  const date = new Date()
  date.setUTCDate(date.getUTCDate() + days)
  return date.toISOString().slice(0, 10)
}

export function statusColor(status: PlanningStatus): string {
  switch (status) {
    case 'active':
      return 'var(--fd-accent)'
    case 'completed':
      return 'var(--fd-success)'
    case 'paused':
      return 'var(--fd-warning)'
    case 'canceled':
      return 'var(--fd-text-muted)'
    default:
      return 'var(--fd-info)'
  }
}

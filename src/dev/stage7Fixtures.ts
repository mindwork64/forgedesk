import type { CommandName, InboxEvent } from '@/types/ipc'

function iso(hoursAgo: number): string {
  return new Date(Date.now() - hoursAgo * 3_600_000).toISOString()
}

const events: InboxEvent[] = [
  {
    id: 'inbox-1',
    source: 'github',
    kind: 'status_changed',
    title: 'Status changed',
    summary: 'Todo → In Progress',
    entityType: 'issue',
    entityId: 'issue-101',
    entityKey: 'FD-141',
    entityTitle: 'GitHub pull request workspace',
    createdAt: iso(1),
    read: false,
    archived: false,
  },
  {
    id: 'inbox-2',
    source: 'local',
    kind: 'comment_added',
    title: 'Comment added',
    summary: 'Design review notes',
    entityType: 'issue',
    entityId: 'issue-102',
    entityKey: 'FD-138',
    entityTitle: 'Repository discovery dialog',
    createdAt: iso(5),
    read: false,
    archived: false,
  },
  {
    id: 'inbox-3',
    source: 'github',
    kind: 'labels_changed',
    title: 'Labels changed',
    summary: 'needs-triage',
    entityType: 'issue',
    entityId: 'issue-103',
    entityKey: 'FD-127',
    entityTitle: 'Sync cursor watermark',
    createdAt: iso(27),
    read: true,
    archived: false,
  },
]

export function handleStage7FixtureCommand<T>(
  command: CommandName,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'inbox_list':
      return { handled: true, value: structuredClone(events) as T }
    case 'inbox_mark_read':
      return { handled: true, value: undefined as T }
    case 'inbox_archive':
      return { handled: true, value: undefined as T }
    case 'inbox_mark_all_read':
      return { handled: true, value: events.filter((event) => !event.read).length as T }
    default:
      return { handled: false }
  }
}

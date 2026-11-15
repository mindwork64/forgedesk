/**
 * Deterministic Stage 3 fixtures for UI Gate screenshots. Build-gated only.
 */
import { issueFixtureList } from './issueFixtures'
import type {
  Cycle,
  CycleDetail,
  CycleInput,
  Document,
  DocumentInput,
  Initiative,
  InitiativeDetail,
  InitiativeInput,
  IssueSummary,
  Milestone,
  MilestoneInput,
  Progress,
  Project,
  ProjectDetail,
  ProjectInput,
  ProjectQuery,
  StatusCount,
} from '@/types/ipc'

const now = Date.now()
const iso = (hoursAgo: number): string =>
  new Date(now - hoursAgo * 3_600_000).toISOString()
const date = (days: number): string => {
  const value = new Date(now)
  value.setDate(value.getDate() + days)
  return value.toISOString().slice(0, 10)
}

function progress(completed: number, total: number, canceled = 0): Progress {
  const eligible = Math.max(0, total - canceled)
  const done = Math.min(completed, eligible)
  return {
    completed: done,
    total: eligible,
    percent: eligible === 0 ? 0 : Math.round((done / eligible) * 1000) / 10,
  }
}

function issueProgress(projectId: string): Progress {
  const issues = issueFixtureList().filter(
    (issue) => issue.project?.id === projectId && !issue.archivedAt,
  )
  return progress(
    issues.filter((issue) => issue.status.category === 'completed').length,
    issues.length,
    issues.filter((issue) => issue.status.category === 'canceled').length,
  )
}

const projectSeeds: Project[] = [
  {
    id: 'project-exhi',
    name: 'EXHI Capture',
    description: 'High-resolution capture pipeline, encoder presets and Wayland support.',
    status: 'active',
    priority: 'high',
    startDate: date(-32),
    targetDate: date(28),
    initiative: { id: 'init-capture', name: 'Capture 2.0' },
    progress: progress(0, 0),
    color: 'accent',
    createdAt: iso(900),
    updatedAt: iso(3),
    archivedAt: null,
  },
  {
    id: 'project-forge',
    name: 'ForgeDesk',
    description: 'Local-first desktop workspace for planning and development.',
    status: 'active',
    priority: 'urgent',
    startDate: date(-60),
    targetDate: date(60),
    initiative: { id: 'init-capture', name: 'Capture 2.0' },
    progress: progress(0, 0),
    color: 'info',
    createdAt: iso(1200),
    updatedAt: iso(7),
    archivedAt: null,
  },
  {
    id: 'project-lab',
    name: 'Lab',
    description: 'Small experiments that do not belong to a product project.',
    status: 'planned',
    priority: 'low',
    startDate: date(10),
    targetDate: date(45),
    initiative: null,
    progress: progress(0, 0),
    color: 'neutral',
    createdAt: iso(400),
    updatedAt: iso(48),
    archivedAt: null,
  },
]

const initiativeSeeds: Initiative[] = [
  {
    id: 'init-tools',
    name: 'Developer tools',
    description: 'Small utilities that keep the daily local workflow fast.',
    status: 'planned',
    priority: 'medium',
    startDate: date(30),
    targetDate: date(90),
    progress: progress(0, 0),
    projectCount: 0,
    createdAt: iso(1400),
    updatedAt: iso(30),
    archivedAt: null,
  },
  {
    id: 'init-capture',
    name: 'Capture 2.0',
    description: 'Ship a stable, high-quality capture experience before the next release.',
    status: 'active',
    priority: 'high',
    startDate: date(-60),
    targetDate: date(45),
    progress: progress(0, 0),
    projectCount: 0,
    createdAt: iso(1600),
    updatedAt: iso(5),
    archivedAt: null,
  },
]

let milestones: Milestone[] = [
  {
    id: 'ms-exhi-beta',
    projectId: 'project-exhi',
    name: 'Capture beta',
    description: 'Beta build with the new encoder.',
    dueDate: date(12),
    status: 'active',
    progress: progress(0, 0),
    createdAt: iso(300),
    updatedAt: iso(20),
  },
  {
    id: 'ms-exhi-ga',
    projectId: 'project-exhi',
    name: 'General availability',
    description: 'Release quality gate.',
    dueDate: date(28),
    status: 'planned',
    progress: progress(0, 0),
    createdAt: iso(280),
    updatedAt: iso(20),
  },
  {
    id: 'ms-forge-stage3',
    projectId: 'project-forge',
    name: 'Stage 3 planning',
    description: 'Projects, initiatives, cycles and documents.',
    dueDate: date(20),
    status: 'active',
    progress: progress(0, 0),
    createdAt: iso(200),
    updatedAt: iso(8),
  },
]

const milestoneIssues = new Map<string, string[]>([
  ['ms-exhi-beta', ['issue-101', 'issue-103']],
  ['ms-exhi-ga', ['issue-109']],
  ['ms-forge-stage3', ['issue-102']],
])

let cycleSeeds: Cycle[] = [
  {
    id: 'cycle-active',
    name: 'Capture hardening',
    startDate: date(-3),
    endDate: date(7),
    status: 'active',
    summary: {
      issueCount: 0,
      completedCount: 0,
      totalEstimate: 0,
      completedEstimate: 0,
      carryOverCount: 1,
      daysRemaining: 7,
    },
    createdAt: iso(240),
    updatedAt: iso(4),
  },
  {
    id: 'cycle-next',
    name: 'Release prep',
    startDate: date(10),
    endDate: date(24),
    status: 'upcoming',
    summary: {
      issueCount: 0,
      completedCount: 0,
      totalEstimate: 0,
      completedEstimate: 0,
      carryOverCount: 0,
      daysRemaining: 24,
    },
    createdAt: iso(220),
    updatedAt: iso(10),
  },
]

const cycleIssues = new Map<string, string[]>([
  ['cycle-active', ['issue-101', 'issue-102', 'issue-103']],
  ['cycle-next', ['issue-109', 'issue-110']],
])

let documents: Document[] = [
  {
    id: 'doc-release',
    title: 'Release notes',
    body: '# Release notes\n\n- Local-first issue workflow\n- Projects and milestones\n- Markdown documents',
    projectId: 'project-forge',
    initiativeId: 'init-capture',
    projectName: 'ForgeDesk',
    initiativeName: 'Capture 2.0',
    archivedAt: null,
    createdAt: iso(120),
    updatedAt: iso(2),
  },
  {
    id: 'doc-capture',
    title: 'Capture quality checklist',
    body: '# Capture quality checklist\n\n1. Verify 4K source\n2. Check encoder preset\n3. Compare frame timing',
    projectId: 'project-exhi',
    initiativeId: null,
    projectName: 'EXHI Capture',
    initiativeName: null,
    archivedAt: null,
    createdAt: iso(90),
    updatedAt: iso(6),
  },
  {
    id: 'doc-roadmap',
    title: 'Stage 3 scope',
    body: '# Stage 3 scope\n\nProjects, initiatives, cycles, roadmap and documents.',
    projectId: null,
    initiativeId: null,
    projectName: null,
    initiativeName: null,
    archivedAt: null,
    createdAt: iso(50),
    updatedAt: iso(12),
  },
]

function projectWithProgress(project: Project): Project {
  return { ...project, progress: issueProgress(project.id) }
}

function allIssues(): IssueSummary[] {
  return issueFixtureList()
}

function milestoneWithProgress(milestone: Milestone): Milestone {
  const ids = milestoneIssues.get(milestone.id) ?? []
  const issues = allIssues().filter((issue) => ids.includes(issue.id))
  return {
    ...milestone,
    progress: progress(
      issues.filter((issue) => issue.status.category === 'completed').length,
      issues.length,
      issues.filter((issue) => issue.status.category === 'canceled').length,
    ),
  }
}

function initiativeWithProgress(initiative: Initiative): Initiative {
  const projects = projectSeeds
    .filter((project) => project.initiative?.id === initiative.id && !project.archivedAt)
    .map(projectWithProgress)
  const completed = projects.reduce(
    (total, project) => total + project.progress.completed,
    0,
  )
  const total = projects.reduce((sum, project) => sum + project.progress.total, 0)
  return {
    ...initiative,
    projectCount: projects.length,
    progress: progress(completed, total),
  }
}

function cycleWithSummary(cycle: Cycle): Cycle {
  const ids = cycleIssues.get(cycle.id) ?? []
  const issues = allIssues().filter((issue) => ids.includes(issue.id))
  const completed = issues.filter(
    (issue) => issue.status.category === 'completed',
  )
  return {
    ...cycle,
    summary: {
      ...cycle.summary,
      issueCount: issues.length,
      completedCount: completed.length,
      totalEstimate: issues.reduce((sum, issue) => sum + (issue.estimate ?? 0), 0),
      completedEstimate: completed.reduce(
        (sum, issue) => sum + (issue.estimate ?? 0),
        0,
      ),
    },
  }
}

function projectIssues(projectId: string): IssueSummary[] {
  return allIssues().filter((issue) => issue.project?.id === projectId)
}

function statusCounts(projectId: string): StatusCount[] {
  const counts = new Map<string, StatusCount>()
  for (const issue of projectIssues(projectId)) {
    const existing = counts.get(issue.status.id)
    if (existing) {
      existing.count += 1
    } else {
      counts.set(issue.status.id, { status: issue.status, count: 1 })
    }
  }
  return [...counts.values()]
}

function projectDetail(project: Project): ProjectDetail {
  return {
    project: projectWithProgress(project),
    milestones: milestones
      .filter((milestone) => milestone.projectId === project.id)
      .map(milestoneWithProgress),
    issueCounts: statusCounts(project.id),
    issueCount: projectIssues(project.id).length,
  }
}

export function handlePlanningFixtureCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'list_project_cards': {
      const query = (args?.query ?? {}) as ProjectQuery
      const needle = query.search?.trim().toLowerCase() ?? ''
      const result = projectSeeds
        .filter((project) => (query.includeArchived ? true : !project.archivedAt))
        .filter((project) => !query.status || project.status === query.status)
        .filter(
          (project) =>
            !query.initiativeId || project.initiative?.id === query.initiativeId,
        )
        .filter((project) => !needle || project.name.toLowerCase().includes(needle))
        .map(projectWithProgress)
      return { handled: true, value: structuredClone(result) as T }
    }
    case 'get_project': {
      const project = projectSeeds.find((item) => item.id === String(args?.id))
      if (!project) {
        throw { code: 'not_found', message: 'Project not found' }
      }
      return { handled: true, value: structuredClone(projectDetail(project)) as T }
    }
    case 'create_project': {
      const input = (args?.input ?? {}) as ProjectInput
      const created: Project = {
        id: `project-${Date.now()}`,
        name: input.name.trim(),
        description: input.description ?? '',
        status: input.status ?? 'planned',
        priority: input.priority ?? 'none',
        startDate: input.startDate ?? null,
        targetDate: input.targetDate ?? null,
        initiative:
          initiativeSeeds.find((item) => item.id === input.initiativeId) ?? null
            ? {
                id: String(input.initiativeId),
                name:
                  initiativeSeeds.find((item) => item.id === input.initiativeId)
                    ?.name ?? '',
              }
            : null,
        progress: progress(0, 0),
        color: input.color ?? 'accent',
        createdAt: iso(0),
        updatedAt: iso(0),
        archivedAt: null,
      }
      if (input.initiativeId) {
        const initiative = initiativeSeeds.find((item) => item.id === input.initiativeId)
        created.initiative = initiative
          ? { id: initiative.id, name: initiative.name }
          : null
      }
      projectSeeds.push(created)
      return { handled: true, value: structuredClone(projectWithProgress(created)) as T }
    }
    case 'update_project': {
      const id = String(args?.id ?? '')
      const index = projectSeeds.findIndex((item) => item.id === id)
      if (index < 0) {
        throw { code: 'not_found', message: 'Project not found' }
      }
      const input = (args?.input ?? {}) as ProjectInput
      const existing = projectSeeds[index]!
      const initiative = input.initiativeId
        ? initiativeSeeds.find((item) => item.id === input.initiativeId) ?? null
        : null
      const updated: Project = {
        ...existing,
        name: input.name.trim(),
        description: input.description ?? '',
        status: input.status ?? existing.status,
        priority: input.priority ?? existing.priority,
        startDate: input.startDate ?? null,
        targetDate: input.targetDate ?? null,
        initiative: initiative
          ? { id: initiative.id, name: initiative.name }
          : null,
        color: input.color ?? existing.color,
        updatedAt: iso(0),
      }
      projectSeeds[index] = updated
      return { handled: true, value: structuredClone(projectWithProgress(updated)) as T }
    }
    case 'archive_project': {
      const index = projectSeeds.findIndex((item) => item.id === String(args?.id))
      if (index < 0) {
        throw { code: 'not_found', message: 'Project not found' }
      }
      const archived = Boolean(args?.archived)
      projectSeeds[index] = {
        ...projectSeeds[index]!,
        archivedAt: archived ? iso(0) : null,
        updatedAt: iso(0),
      }
      return {
        handled: true,
        value: structuredClone(projectWithProgress(projectSeeds[index]!)) as T,
      }
    }
    case 'create_milestone': {
      const projectId = String(args?.projectId ?? '')
      const input = (args?.input ?? {}) as MilestoneInput
      const created: Milestone = {
        id: `ms-${Date.now()}`,
        projectId,
        name: input.name.trim(),
        description: input.description ?? '',
        dueDate: input.dueDate ?? null,
        status: input.status ?? 'planned',
        progress: progress(0, 0),
        createdAt: iso(0),
        updatedAt: iso(0),
      }
      milestones.push(created)
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'update_milestone': {
      const index = milestones.findIndex((item) => item.id === String(args?.id))
      if (index < 0) {
        throw { code: 'not_found', message: 'Milestone not found' }
      }
      const input = (args?.input ?? {}) as MilestoneInput
      milestones[index] = {
        ...milestones[index]!,
        name: input.name.trim(),
        description: input.description ?? '',
        dueDate: input.dueDate ?? null,
        status: input.status ?? milestones[index]!.status,
        updatedAt: iso(0),
      }
      return {
        handled: true,
        value: structuredClone(milestoneWithProgress(milestones[index]!)) as T,
      }
    }
    case 'assign_milestone_issue': {
      const milestoneId = String(args?.milestoneId ?? '')
      const issueId = String(args?.issueId ?? '')
      const ids = milestoneIssues.get(milestoneId) ?? []
      if (args?.assigned) {
        if (!ids.includes(issueId)) {
          ids.push(issueId)
        }
      } else {
        milestoneIssues.set(
          milestoneId,
          ids.filter((id) => id !== issueId),
        )
        return { handled: true, value: undefined as T }
      }
      milestoneIssues.set(milestoneId, ids)
      return { handled: true, value: undefined as T }
    }
    case 'list_initiatives':
      return {
        handled: true,
        value: structuredClone(
          initiativeSeeds.map(initiativeWithProgress),
        ) as T,
      }
    case 'get_initiative': {
      const initiative = initiativeSeeds.find((item) => item.id === String(args?.id))
      if (!initiative) {
        throw { code: 'not_found', message: 'Initiative not found' }
      }
      const detail: InitiativeDetail = {
        initiative: initiativeWithProgress(initiative),
        projects: projectSeeds
          .filter((project) => project.initiative?.id === initiative.id)
          .map(projectWithProgress),
      }
      return { handled: true, value: structuredClone(detail) as T }
    }
    case 'create_initiative': {
      const input = (args?.input ?? {}) as InitiativeInput
      const created: Initiative = {
        id: `init-${Date.now()}`,
        name: input.name.trim(),
        description: input.description ?? '',
        status: input.status ?? 'planned',
        priority: input.priority ?? 'none',
        startDate: input.startDate ?? null,
        targetDate: input.targetDate ?? null,
        progress: progress(0, 0),
        projectCount: 0,
        createdAt: iso(0),
        updatedAt: iso(0),
        archivedAt: null,
      }
      initiativeSeeds.push(created)
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'update_initiative': {
      const index = initiativeSeeds.findIndex((item) => item.id === String(args?.id))
      if (index < 0) {
        throw { code: 'not_found', message: 'Initiative not found' }
      }
      const input = (args?.input ?? {}) as InitiativeInput
      initiativeSeeds[index] = {
        ...initiativeSeeds[index]!,
        name: input.name.trim(),
        description: input.description ?? '',
        status: input.status ?? initiativeSeeds[index]!.status,
        priority: input.priority ?? initiativeSeeds[index]!.priority,
        startDate: input.startDate ?? null,
        targetDate: input.targetDate ?? null,
        updatedAt: iso(0),
      }
      return {
        handled: true,
        value: structuredClone(initiativeWithProgress(initiativeSeeds[index]!)) as T,
      }
    }
    case 'list_cycles':
      return {
        handled: true,
        value: structuredClone(cycleSeeds.map(cycleWithSummary)) as T,
      }
    case 'get_cycle': {
      const cycle = cycleSeeds.find((item) => item.id === String(args?.id))
      if (!cycle) {
        throw { code: 'not_found', message: 'Cycle not found' }
      }
      const ids = cycleIssues.get(cycle.id) ?? []
      const detail: CycleDetail = {
        cycle: cycleWithSummary(cycle),
        issues: allIssues().filter((issue) => ids.includes(issue.id)),
      }
      return { handled: true, value: structuredClone(detail) as T }
    }
    case 'create_cycle': {
      const input = (args?.input ?? {}) as CycleInput
      const created: Cycle = {
        id: `cycle-${Date.now()}`,
        name: input.name.trim(),
        startDate: input.startDate,
        endDate: input.endDate,
        status: 'upcoming',
        summary: {
          issueCount: 0,
          completedCount: 0,
          totalEstimate: 0,
          completedEstimate: 0,
          carryOverCount: 0,
          daysRemaining: 0,
        },
        createdAt: iso(0),
        updatedAt: iso(0),
      }
      cycleSeeds.push(created)
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'update_cycle': {
      const index = cycleSeeds.findIndex((item) => item.id === String(args?.id))
      if (index < 0) {
        throw { code: 'not_found', message: 'Cycle not found' }
      }
      const input = (args?.input ?? {}) as CycleInput
      cycleSeeds[index] = {
        ...cycleSeeds[index]!,
        name: input.name.trim(),
        startDate: input.startDate,
        endDate: input.endDate,
        updatedAt: iso(0),
      }
      return {
        handled: true,
        value: structuredClone(cycleWithSummary(cycleSeeds[index]!)) as T,
      }
    }
    case 'assign_cycle_issue': {
      const cycleId = String(args?.cycleId ?? '')
      const issueId = String(args?.issueId ?? '')
      const ids = cycleIssues.get(cycleId) ?? []
      if (args?.assigned) {
        if (!ids.includes(issueId)) {
          ids.push(issueId)
        }
      } else {
        cycleIssues.set(
          cycleId,
          ids.filter((id) => id !== issueId),
        )
        return { handled: true, value: undefined as T }
      }
      cycleIssues.set(cycleId, ids)
      return { handled: true, value: undefined as T }
    }
    case 'list_documents':
      return { handled: true, value: structuredClone(documents) as T }
    case 'get_document': {
      const document = documents.find((item) => item.id === String(args?.id))
      if (!document) {
        throw { code: 'not_found', message: 'Document not found' }
      }
      return { handled: true, value: structuredClone(document) as T }
    }
    case 'create_document': {
      const input = (args?.input ?? {}) as DocumentInput
      const project = projectSeeds.find((item) => item.id === input.projectId)
      const initiative = initiativeSeeds.find((item) => item.id === input.initiativeId)
      const created: Document = {
        id: `doc-${Date.now()}`,
        title: input.title.trim(),
        body: input.body ?? '',
        projectId: input.projectId ?? null,
        initiativeId: input.initiativeId ?? null,
        projectName: project?.name ?? null,
        initiativeName: initiative?.name ?? null,
        archivedAt: null,
        createdAt: iso(0),
        updatedAt: iso(0),
      }
      documents = [created, ...documents]
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'update_document': {
      const index = documents.findIndex((item) => item.id === String(args?.id))
      if (index < 0) {
        throw { code: 'not_found', message: 'Document not found' }
      }
      const input = (args?.input ?? {}) as DocumentInput
      const project = projectSeeds.find((item) => item.id === input.projectId)
      const initiative = initiativeSeeds.find((item) => item.id === input.initiativeId)
      const updated: Document = {
        ...documents[index]!,
        title: input.title.trim(),
        body: input.body ?? '',
        projectId: input.projectId ?? null,
        initiativeId: input.initiativeId ?? null,
        projectName: project?.name ?? null,
        initiativeName: initiative?.name ?? null,
        updatedAt: iso(0),
      }
      documents[index] = updated
      return { handled: true, value: structuredClone(updated) as T }
    }
    case 'delete_document':
      documents = documents.filter((item) => item.id !== String(args?.id))
      return { handled: true, value: undefined as T }
    case 'export_document': {
      const document = documents.find((item) => item.id === String(args?.id))
      return {
        handled: true,
        value: (document ? `# ${document.title}\n\n${document.body}\n` : '') as T,
      }
    }
    default:
      return { handled: false }
  }
}

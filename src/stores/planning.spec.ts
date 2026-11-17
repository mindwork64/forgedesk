import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { handlePlanningFixtureCommand } from '@/dev/planningFixtures'
import { setDevTransport } from '@/lib/ipc'
import { usePlanningStore } from './planning'

describe('planning store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    setDevTransport(
      async <T,>(
        command: Parameters<typeof handlePlanningFixtureCommand>[0],
        args?: Record<string, unknown>,
      ) => {
        const result = handlePlanningFixtureCommand<T>(command, args)
        if (!result.handled) {
          throw { code: 'unsupported_capability', message: String(command) }
        }
        return result.value as T
      },
    )
  })

  it('loads projects and their overview', async () => {
    const store = usePlanningStore()
    await store.loadProjects()
    expect(store.projects.length).toBeGreaterThanOrEqual(3)
    const project = store.projects.find((item) => item.id === 'project-exhi')
    expect(project).toBeTruthy()
    expect(project?.progress.total).toBeGreaterThan(0)

    await store.loadProject('project-exhi')
    expect(store.project?.project.id).toBe('project-exhi')
    expect(store.project?.milestones.length).toBeGreaterThan(0)
    expect(store.project?.issueCounts.length).toBeGreaterThan(0)
  })

  it('removes archived projects from the active list and restores them', async () => {
    const store = usePlanningStore()
    await store.loadProjects()
    const project = store.projects[0]
    expect(project).toBeTruthy()
    setDevTransport(async <T,>(
      command: Parameters<typeof handlePlanningFixtureCommand>[0],
      args?: Record<string, unknown>,
    ) => {
      if (command === 'archive_project') {
        return {
          ...project!,
          archivedAt: (args?.archived as boolean) ? '2026-11-17T00:00:00.000Z' : null,
        } as T
      }
      const result = handlePlanningFixtureCommand<T>(command, args)
      if (!result.handled) throw { code: 'unsupported_capability', message: String(command) }
      return result.value as T
    })

    expect(await store.archiveProject(project!.id, true)).toBe(true)
    expect(store.projects.some((item) => item.id === project!.id)).toBe(false)
    expect(await store.archiveProject(project!.id, false)).toBe(true)
    expect(store.projects.some((item) => item.id === project!.id)).toBe(true)
  })

  it('creates a project and loads initiatives/cycles/documents', async () => {
    const store = usePlanningStore()
    await store.loadProjects()
    const created = await store.createProject({ name: 'Store project' })
    expect(created).not.toBeNull()
    expect(store.projects.some((item) => item.id === created?.id)).toBe(true)

    await store.loadInitiatives()
    expect(store.initiatives.length).toBeGreaterThanOrEqual(2)
    await store.loadInitiative('init-capture')
    expect(store.initiative?.projects.length).toBeGreaterThan(0)

    await store.loadCycles()
    expect(store.cycles.length).toBeGreaterThanOrEqual(2)
    await store.loadCycle('cycle-active')
    expect(store.cycle?.issues.length).toBeGreaterThan(0)

    await store.loadDocuments()
    expect(store.documents.length).toBeGreaterThanOrEqual(3)
    const document = await store.createDocument({
      title: 'Store document',
      body: '# Store\n\nBody.',
    })
    expect(document).not.toBeNull()
    const markdown = await store.exportDocument(document!.id)
    expect(markdown).toContain('# Store document')
  })
})

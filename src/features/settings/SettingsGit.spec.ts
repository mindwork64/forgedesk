import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import type { GitAvailability, Repository } from '@/types/ipc'
import SettingsGit from './SettingsGit.vue'

const timestamp = '2026-11-15T12:00:00.000Z'

function repository(id: string, name: string): Repository {
  return {
    id,
    provider: 'local',
    host: 'local',
    externalId: null,
    owner: null,
    name,
    fullName: name,
    description: null,
    visibility: null,
    defaultBranch: 'main',
    htmlUrl: null,
    cloneUrl: null,
    canonicalRemoteUrl: null,
    isArchived: false,
    identityConflict: null,
    checkouts: [
      {
        id: `checkout-${id}`,
        repositoryId: id,
        path: `/tmp/${name}`,
        lastCheckedAt: null,
        createdAt: timestamp,
        updatedAt: timestamp,
      },
    ],
    projects: [],
    lastCheckedAt: null,
    createdAt: timestamp,
    updatedAt: timestamp,
  }
}

function availability(version = 'git version 2.48.1'): GitAvailability {
  return { available: true, version, error: null }
}

describe('SettingsGit', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let currentAvailability: GitAvailability
  let currentRepositories: Repository[]
  let availabilityCalls = 0
  let repositoryCalls = 0
  let router: ReturnType<typeof createRouter>

  async function mountGit(
    initial: GitAvailability,
    repositories: Repository[],
    locale: 'en' | 'ru' = 'en',
  ): Promise<void> {
    setActivePinia(createPinia())
    currentAvailability = initial
    currentRepositories = repositories
    availabilityCalls = 0
    repositoryCalls = 0
    i18n.global.locale.value = locale
    setDevTransport(async <T,>(command: string) => {
      if (command === 'git_available') {
        availabilityCalls += 1
        return structuredClone(currentAvailability) as T
      }
      if (command === 'list_repositories') {
        repositoryCalls += 1
        return structuredClone(currentRepositories) as T
      }
      throw { code: 'unsupported_capability', message: String(command) }
    })
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/settings/git', component: { template: '<div />' } },
        { path: '/repositories', component: { template: '<div />' } },
      ],
    })
    await router.push('/settings/git')
    await router.isReady()
    wrapper = mount(SettingsGit, {
      global: { plugins: [router, i18n] },
      attachTo: document.body,
    })
    await flushPromises()
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('shows Git version and repository count when available', async () => {
    await mountGit(availability(), [repository('r1', 'one'), repository('r2', 'two')])
    expect(wrapper?.text()).toContain('Git available')
    expect(wrapper?.text()).toContain('git version 2.48.1')
    expect(wrapper?.find('.fd-git-count').text()).toBe('2')
    expect(availabilityCalls).toBe(1)
    expect(repositoryCalls).toBe(1)
  })

  it('distinguishes missing Git from a failed availability check', async () => {
    await mountGit(
      { available: false, version: null, error: 'Git is not available: not found' },
      [],
    )
    expect(wrapper?.text()).toContain('Git not found')
    expect(wrapper?.text()).not.toContain('Availability check failed')
    wrapper?.unmount()

    await mountGit(
      { available: false, version: null, error: 'Git command failed.' },
      [],
    )
    expect(wrapper?.text()).toContain('Availability check failed')
    expect(wrapper?.text()).not.toContain('Git not found')
  })

  it('shows zero repositories and still offers management', async () => {
    await mountGit(availability(), [])
    expect(wrapper?.find('.fd-git-count').text()).toBe('0')
    expect(wrapper?.text()).toContain('Manage repositories')
  })

  it('reloads availability and repositories', async () => {
    await mountGit(availability('git version 2.48.1'), [repository('r1', 'one')])
    currentAvailability = availability('git version 2.49.0')
    currentRepositories = [repository('r1', 'one'), repository('r2', 'two')]
    const refresh = wrapper
      ?.findAll('button')
      .find((button) => button.text().includes('Check again'))
    await refresh?.trigger('click')
    await flushPromises()
    expect(availabilityCalls).toBe(2)
    expect(repositoryCalls).toBe(2)
    expect(wrapper?.text()).toContain('git version 2.49.0')
    expect(wrapper?.find('.fd-git-count').text()).toBe('2')
  })

  it('shows localized Git availability and actions in Russian', async () => {
    await mountGit(availability(), [repository('r1', 'one')], 'ru')
    expect(wrapper?.text()).toContain('Git доступен')
    expect(wrapper?.text()).toContain('Управлять репозиториями')
    expect(wrapper?.text()).toContain('Проверить снова')
  })

  it('navigates to the existing repositories screen', async () => {
    await mountGit(availability(), [])
    const manage = wrapper
      ?.findAll('button')
      .find((button) => button.text().includes('Manage repositories'))
    await manage?.trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/repositories')
  })
})

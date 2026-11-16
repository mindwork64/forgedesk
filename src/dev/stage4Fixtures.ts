/** Stage 4 local Git/API budget fixtures for visual gates. Build-gated only. */
import type {
  ApiControl,
  ApiMonitorState,
  BudgetSummary,
  GitAvailability,
  GitDiffStat,
  GitLogEntry,
  GitHubConnection,
  GitHubImportResult,
  GitOverview,
  GitStatus,
  QueueJob,
  Repository,
  RequestLog,
} from '@/types/ipc'

const now = Date.now()
const iso = (minutesAgo: number): string =>
  new Date(now - minutesAgo * 60_000).toISOString()

function checkout(id: string, repositoryId: string, path: string, checked: number) {
  return {
    id,
    repositoryId,
    path,
    lastCheckedAt: iso(checked),
    createdAt: iso(checked + 100),
    updatedAt: iso(checked),
  }
}

const repositories: Repository[] = [
  {
    id: 'repo-forge',
    provider: 'local',
    host: 'local',
    externalId: null,
    owner: 'eloraam',
    name: 'ForgeDesk',
    fullName: 'eloraam/ForgeDesk',
    description: 'Local-first desktop workspace',
    visibility: null,
    defaultBranch: 'main',
    htmlUrl: 'https://github.com/eloraam/forgedesk',
    cloneUrl: 'git@github.com:eloraam/forgedesk.git',
    canonicalRemoteUrl: 'https://github.com/eloraam/forgedesk',
    isArchived: false,
    identityConflict: null,
    checkouts: [checkout('checkout-forge', 'repo-forge', '/home/dev/code/forgedesk', 4)],
    projects: [{ id: 'project-forge', name: 'ForgeDesk' }],
    lastCheckedAt: iso(4),
    createdAt: iso(2400),
    updatedAt: iso(4),
  },
  {
    id: 'repo-remote',
    provider: 'github',
    host: 'github.com',
    externalId: '9876',
    owner: 'mindwork64',
    name: 'remote-only',
    fullName: 'mindwork64/remote-only',
    description: 'Remote repository without a local clone',
    visibility: 'public',
    defaultBranch: 'main',
    htmlUrl: 'https://github.com/mindwork64/remote-only',
    cloneUrl: 'https://github.com/mindwork64/remote-only.git',
    canonicalRemoteUrl: 'https://github.com/mindwork64/remote-only',
    isArchived: false,
    identityConflict: null,
    checkouts: [],
    projects: [],
    lastCheckedAt: null,
    createdAt: iso(5000),
    updatedAt: iso(30),
  },
  {
    id: 'repo-exhi',
    provider: 'local',
    host: 'local',
    externalId: null,
    owner: 'eloraam',
    name: 'EXHI Capture',
    fullName: 'eloraam/EXHI Capture',
    description: 'Capture pipeline',
    visibility: null,
    defaultBranch: 'main',
    htmlUrl: null,
    cloneUrl: 'git@github.com:eloraam/exhi-capture.git',
    canonicalRemoteUrl: 'https://github.com/eloraam/exhi-capture',
    isArchived: false,
    identityConflict: null,
    checkouts: [checkout('checkout-exhi', 'repo-exhi', '/home/dev/code/exhi capture', 18)],
    projects: [{ id: 'project-exhi', name: 'EXHI Capture' }],
    lastCheckedAt: iso(18),
    createdAt: iso(4000),
    updatedAt: iso(18),
  },
]

const discoveredRepositories: Repository[] = repositories.map((repository) => ({
  ...repository,
  provider: 'github',
  host: 'github.com',
  externalId: `gh-${repository.id}`,
  visibility: 'public',
  htmlUrl: repository.htmlUrl ?? 'https://github.com/mindwork64/forgedesk',
  cloneUrl: repository.cloneUrl ?? 'https://github.com/mindwork64/forgedesk.git',
  canonicalRemoteUrl:
    repository.canonicalRemoteUrl ?? 'https://github.com/mindwork64/forgedesk',
  checkouts: [],
}))

const selectedRepository: Repository = {
  id: 'selected-gh',
  provider: 'github',
  host: 'github.com',
  externalId: '1001',
  owner: 'mindwork64',
  name: 'forgedesk',
  fullName: 'mindwork64/forgedesk',
  description: 'Local-first desktop workspace',
  visibility: 'public',
  defaultBranch: 'main',
  htmlUrl: 'https://github.com/mindwork64/forgedesk',
  cloneUrl: 'https://github.com/mindwork64/forgedesk.git',
  canonicalRemoteUrl: 'https://github.com/mindwork64/forgedesk',
  isArchived: false,
  identityConflict: null,
  checkouts: [],
  projects: [],
  lastCheckedAt: null,
  createdAt: iso(6000),
  updatedAt: iso(30),
}

const statuses: Record<string, GitStatus> = {
  'repo-forge': {
    branch: 'main',
    upstream: 'origin/main',
    ahead: 2,
    behind: 0,
    dirty: false,
    changes: [],
  },
  'repo-exhi': {
    branch: 'fix/encoder',
    upstream: 'origin/fix/encoder',
    ahead: 1,
    behind: 1,
    dirty: true,
    changes: [' M src/encode/preset.rs', '?? notes.txt'],
  },
}

const branches: Record<string, string[]> = {
  'repo-forge': ['main', 'feature/filters', 'fix/palette'],
  'repo-exhi': ['main', 'fix/encoder', 'feature/quality'],
}

const logs: Record<string, GitLogEntry[]> = {
  'repo-forge': [
    {
      hash: 'a1b2c3d4e5f6',
      shortHash: 'a1b2c3d',
      subject: 'Stage 3 planning surfaces',
      author: 'Eloraam',
      committedAt: iso(180),
    },
    {
      hash: 'b2c3d4e5f6a1',
      shortHash: 'b2c3d4e',
      subject: 'Fix saved view restore',
      author: 'Eloraam',
      committedAt: iso(420),
    },
    {
      hash: 'c3d4e5f6a1b2',
      shortHash: 'c3d4e5f',
      subject: 'Visual gate baseline refresh',
      author: 'Eloraam',
      committedAt: iso(1440),
    },
  ],
  'repo-exhi': [
    {
      hash: 'd4e5f6a1b2c3',
      shortHash: 'd4e5f6a',
      subject: 'Tune encoder preset',
      author: 'Eloraam',
      committedAt: iso(90),
    },
    {
      hash: 'e5f6a1b2c3d4',
      shortHash: 'e5f6a1b',
      subject: 'Wayland capture fallback',
      author: 'Eloraam',
      committedAt: iso(600),
    },
  ],
}

const diffs: Record<string, GitDiffStat> = {
  'repo-forge': {
    filesChanged: 0,
    insertions: 0,
    deletions: 0,
    summary: '',
  },
  'repo-exhi': {
    filesChanged: 1,
    insertions: 12,
    deletions: 4,
    summary: ' src/encode/preset.rs | 16 ++++++++++++----\n 1 file changed, 12 insertions(+), 4 deletions(-)',
  },
}

const budgets: BudgetSummary[] = [
  {
    apiFamily: 'github_graphql',
    resourceName: 'graphql',
    limitValue: 5000,
    remainingValue: 4210,
    usedValue: 790,
    resetAtUtc: iso(-42),
    observedAtUtc: iso(3),
    httpStatus: 200,
    source: 'response_headers',
    isEstimate: false,
    secondaryPauseUntil: null,
    backgroundAllowance: 500,
    backgroundReserve: 1000,
    observedRequests: 38,
  },
  {
    apiFamily: 'github_rest',
    resourceName: 'core',
    limitValue: 5000,
    remainingValue: 4770,
    usedValue: 230,
    resetAtUtc: iso(-42),
    observedAtUtc: iso(3),
    httpStatus: 200,
    source: 'response_headers',
    isEstimate: false,
    secondaryPauseUntil: null,
    backgroundAllowance: 500,
    backgroundReserve: 1000,
    observedRequests: 11,
  },
  {
    apiFamily: 'github_search',
    resourceName: 'search',
    limitValue: 30,
    remainingValue: 27,
    usedValue: 3,
    resetAtUtc: iso(-42),
    observedAtUtc: iso(3),
    httpStatus: 200,
    source: 'response_headers',
    isEstimate: false,
    secondaryPauseUntil: null,
    backgroundAllowance: 3,
    backgroundReserve: 6,
    observedRequests: 1,
  },
]

const requests: RequestLog[] = [
  {
    id: 'req-1',
    providerAccountId: 'acct-1',
    operationName: 'IssueList',
    operationCode: 'issues.list',
    apiFamily: 'github_graphql',
    resourceName: 'graphql',
    requestClass: 'interactive_read',
    fingerprint: 'sha256:3f5a…',
    estimatedCost: 1,
    actualCost: 1,
    httpStatus: 200,
    errorCategory: null,
    latencyMs: 240,
    receivedAtUtc: iso(3),
    retryAttempt: 0,
    resultState: 'success',
    rateHeadersJson: '{}',
    payloadBytes: 8124,
  },
  {
    id: 'req-2',
    providerAccountId: 'acct-1',
    operationName: 'RepositoryList',
    operationCode: 'repos.list',
    apiFamily: 'github_rest',
    resourceName: 'core',
    requestClass: 'foreground_refresh',
    fingerprint: 'sha256:8b21…',
    estimatedCost: 1,
    actualCost: 1,
    httpStatus: 304,
    errorCategory: null,
    latencyMs: 92,
    receivedAtUtc: iso(18),
    retryAttempt: 0,
    resultState: 'not_modified',
    rateHeadersJson: '{}',
    payloadBytes: 0,
  },
  {
    id: 'req-3',
    providerAccountId: 'acct-1',
    operationName: 'IssueSearch',
    operationCode: 'search.issues',
    apiFamily: 'github_search',
    resourceName: 'search',
    requestClass: 'background_sync',
    fingerprint: 'sha256:c901…',
    estimatedCost: 1,
    actualCost: 1,
    httpStatus: 200,
    errorCategory: null,
    latencyMs: 310,
    receivedAtUtc: iso(44),
    retryAttempt: 1,
    resultState: 'success',
    rateHeadersJson: '{}',
    payloadBytes: 4096,
  },
]

const queue: QueueJob[] = [
  {
    id: 'job-1',
    operationName: 'ProjectIssuesRefresh',
    operationCode: 'issues.project',
    apiFamily: 'github_graphql',
    priority: 2,
    requestClass: 'foreground_refresh',
    state: 'queued',
    attempts: 0,
    estimatedCost: 2,
    fingerprint: 'sha256:aa11…',
    createdAt: iso(2),
    updatedAt: iso(2),
    lastError: null,
  },
  {
    id: 'job-2',
    operationName: 'TrackedIssuesSync',
    operationCode: 'issues.sync',
    apiFamily: 'github_graphql',
    priority: 3,
    requestClass: 'background_sync',
    state: 'paused',
    attempts: 1,
    estimatedCost: 5,
    fingerprint: 'sha256:bb22…',
    createdAt: iso(6),
    updatedAt: iso(4),
    lastError: null,
  },
]

let control: ApiControl = {
  paused: false,
  reason: '',
  updatedAt: iso(4),
}

let githubConnected = true
const githubConnection: GitHubConnection = {
  connected: true,
  login: 'eloraam',
  userId: 34349377,
  scopes: '',
  expiresAt: iso(-420),
  refreshExpiresAt: iso(-14000),
  storage: 'keyring',
  storageWarning: null,
}

function overviewFor(id: string): GitOverview {
  const repository = repositories.find((item) => item.id === id)
  if (!repository) {
    throw { code: 'not_found', message: 'Repository not found' }
  }
  return {
    repository,
    status: statuses[id] ?? {
      branch: 'main',
      upstream: null,
      ahead: 0,
      behind: 0,
      dirty: false,
      changes: [],
    },
    branches: (branches[id] ?? ['main']).map((name) => ({
      name,
      current: name === (statuses[id]?.branch ?? 'main'),
    })),
    log: logs[id] ?? [],
    diff: diffs[id] ?? {
      filesChanged: 0,
      insertions: 0,
      deletions: 0,
      summary: '',
    },
  }
}

function monitor(): ApiMonitorState {
  return {
    connected: true,
    accountLogin: 'eloraam',
    budgets: structuredClone(budgets),
    requests: structuredClone(requests),
    queue: structuredClone(queue),
    control: { ...control },
    lastObservation: iso(3),
  }
}

export function handleStage4FixtureCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'git_available':
      return {
        handled: true,
        value: {
          available: true,
          version: 'git version 2.48.1',
          error: null,
        } satisfies GitAvailability as T,
      }
    case 'list_repositories':
      return { handled: true, value: structuredClone(repositories) as T }
    case 'add_local_repository': {
      const input = (args?.input ?? {}) as { path?: string; name?: string; projectId?: string }
      const path = input.path ?? '/home/dev/code/added'
      const id = `repo-${Date.now()}`
      const name = input.name?.trim() || path.split('/').pop() || 'repository'
      const created: Repository = {
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
        checkouts: [checkout(`checkout-${id}`, id, path, 0)],
        projects: input.projectId ? [{ id: input.projectId, name: input.projectId }] : [],
        lastCheckedAt: iso(0),
        createdAt: iso(0),
        updatedAt: iso(0),
      }
      repositories.push(created)
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'remove_repository': {
      const index = repositories.findIndex((item) => item.id === String(args?.id))
      if (index >= 0) {
        repositories.splice(index, 1)
      }
      return { handled: true, value: undefined as T }
    }
    case 'get_git_overview':
      return {
        handled: true,
        value: structuredClone(overviewFor(String(args?.id))) as T,
      }
    case 'create_git_branch': {
      const id = String(args?.id)
      const branch = String(args?.branch ?? '')
      if (!branches[id]) {
        branches[id] = ['main']
      }
      if (!branches[id]!.includes(branch)) {
        branches[id]!.push(branch)
      }
      statuses[id] = {
        ...(statuses[id] ?? {
          upstream: null,
          ahead: 0,
          behind: 0,
          dirty: false,
          changes: [],
        }),
        branch,
        dirty: false,
        changes: [],
      }
      return { handled: true, value: structuredClone(overviewFor(id)) as T }
    }
    case 'switch_git_branch': {
      const id = String(args?.id)
      const branch = String(args?.branch ?? '')
      if (!branches[id]?.includes(branch)) {
        throw { code: 'validation', message: 'Branch does not exist.' }
      }
      statuses[id] = {
        ...(statuses[id] ?? {
          upstream: null,
          ahead: 0,
          behind: 0,
          dirty: false,
          changes: [],
        }),
        branch,
      }
      return { handled: true, value: structuredClone(overviewFor(id)) as T }
    }
    case 'get_git_log':
      return { handled: true, value: structuredClone(logs[String(args?.id)] ?? []) as T }
    case 'get_git_diff':
      return {
        handled: true,
        value: structuredClone(
          diffs[String(args?.id)] ?? {
            filesChanged: 0,
            insertions: 0,
            deletions: 0,
            summary: '',
          },
        ) as T,
      }
    case 'github_connection':
      return {
        handled: true,
        value: (githubConnected ? structuredClone(githubConnection) : null) as T,
      }
    case 'github_start_device_flow':
      return {
        handled: true,
        value: {
          userCode: 'TEST-CODE',
          verificationUri: 'https://github.com/login/device',
          expiresIn: 899,
          interval: 5,
        } as T,
      }
    case 'github_poll_device_flow':
      githubConnected = true
      return {
        handled: true,
        value: {
          status: 'success',
          interval: 5,
          connection: structuredClone(githubConnection),
        } as T,
      }
    case 'github_refresh':
      return { handled: true, value: structuredClone(githubConnection) as T }
    case 'github_disconnect':
      githubConnected = false
      return { handled: true, value: undefined as T }
    case 'get_repository':
      return {
        handled: true,
        value: structuredClone(
          repositories.find((repository) => repository.id === String(args?.id)) ?? repositories[0]!,
        ) as T,
      }
    case 'get_project_repositories':
      return {
        handled: true,
        value: structuredClone(
          repositories.filter((repository) =>
            repository.projects.some((project) => project.id === String(args?.projectId)),
          ),
        ) as T,
      }
    case 'link_repository': {
      const repository = repositories.find((item) => item.id === String(args?.repositoryId))
      if (repository && !repository.projects.some((project) => project.id === String(args?.projectId))) {
        repository.projects.push({ id: String(args?.projectId), name: String(args?.projectId) })
      }
      return { handled: true, value: structuredClone(repository) as T }
    }
    case 'unlink_repository': {
      const repository = repositories.find((item) => item.id === String(args?.repositoryId))
      if (repository) {
        repository.projects = repository.projects.filter(
          (project) => project.id !== String(args?.projectId),
        )
      }
      return { handled: true, value: structuredClone(repository) as T }
    }
    case 'selected_repository':
      return { handled: true, value: structuredClone(selectedRepository) as T }
    case 'select_repository':
      return { handled: true, value: structuredClone(selectedRepository) as T }
    case 'github_discover_repositories':
      return {
        handled: true,
        value: {
          items: structuredClone(discoveredRepositories),
          hasMore: false,
          pagesLoaded: 1,
          nextPage: null,
        } as T,
      }
    case 'github_import_issues':
      return {
        handled: true,
        value: {
          imported: 5,
          skipped: 2,
          projectId: 'project-forge',
          projectName: 'mindwork64/forgedesk',
          repositoryId: 'repo-forge',
          hasMore: false,
          pages: 1,
          nextPage: null,
        } satisfies GitHubImportResult as T,
      }
    case 'get_api_monitor':
      return { handled: true, value: monitor() as T }
    case 'set_api_paused':
      control = {
        paused: Boolean(args?.paused),
        reason: String(args?.reason ?? ''),
        updatedAt: iso(0),
      }
      return { handled: true, value: { ...control } as T }
    case 'export_api_diagnostics':
      return { handled: true, value: JSON.stringify(monitor(), null, 2) as T }
    default:
      return { handled: false }
  }
}

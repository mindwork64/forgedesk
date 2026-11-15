import type {
  CommandName,
  GitHubCheckRun,
  GitHubCommit,
  GitHubPull,
  GitHubReview,
  GitHubFile,
  GitHubSearchIssue,
  CiOverview,
  GitHubReviewComment,
  PullLink,
  PullCapabilities,
  GitHubMergeResult,
  PullRequestDetail,
} from '@/types/ipc'

function iso(hoursAgo: number): string {
  return new Date(Date.now() - hoursAgo * 3_600_000).toISOString()
}

const pullOne: GitHubPull = {
  id: 601,
  number: 214,
  title: 'Add repository discovery for GitHub App accounts',
  state: 'open',
  draft: false,
  user: { login: 'eloraam' },
  head: { ref: 'feat/repo-discovery', sha: 'a1b2c3d4e5f60718293a4b5c6d7e8f9012345678' },
  base: { ref: 'main', sha: 'b2c3d4e5f60718293a4b5c6d7e8f9012345678a1' },
  updatedAt: iso(2),
  createdAt: iso(26),
  htmlUrl: 'https://github.com/mindwork64/forgedesk/pull/214',
  body: 'Adds the GitHub repository picker and read-only issue import.\n\n- device flow auth\n- repository discovery\n- idempotent remote links',
  mergeable: true,
  mergedAt: null,
  additions: 284,
  deletions: 41,
  changedFiles: 9,
  commits: 4,
  requestedReviewers: [{ login: 'mindwork64' }],
}

const pullTwo: GitHubPull = {
  id: 602,
  number: 209,
  title: 'Stabilize API Monitor budget observations',
  state: 'open',
  draft: true,
  user: { login: 'mindwork64' },
  head: { ref: 'fix/budget-observations', sha: 'c3d4e5f60718293a4b5c6d7e8f9012345678a1b2' },
  base: { ref: 'main', sha: 'b2c3d4e5f60718293a4b5c6d7e8f9012345678a1' },
  updatedAt: iso(5),
  createdAt: iso(50),
  htmlUrl: 'https://github.com/mindwork64/forgedesk/pull/209',
  body: 'Keeps the observed-request count separate from the reset window.',
  mergeable: null,
  mergedAt: null,
  additions: 92,
  deletions: 18,
  changedFiles: 4,
  commits: 2,
  requestedReviewers: [],
}

const commits: GitHubCommit[] = [
  {
    sha: 'a1b2c3d4e5f60718293a4b5c6d7e8f9012345678',
    commit: { message: 'Add repository discovery dialog', author: { name: 'Vitaly Murashov', date: iso(4) } },
    author: { login: 'eloraam' },
  },
  {
    sha: 'd4e5f60718293a4b5c6d7e8f9012345678a1b2c3',
    commit: { message: 'Wire read-only issue import', author: { name: 'Vitaly Murashov', date: iso(7) } },
    author: { login: 'eloraam' },
  },
  {
    sha: 'e5f60718293a4b5c6d7e8f9012345678a1b2c3d4',
    commit: { message: 'Add import fixtures and gate capture', author: { name: 'Vitaly Murashov', date: iso(11) } },
    author: { login: 'eloraam' },
  },
]

const reviews: GitHubReview[] = [
  {
    id: 801,
    user: { login: 'mindwork64' },
    state: 'APPROVED',
    body: 'Looks good; keep import read-only until writes are reconciled.',
    submittedAt: iso(3),
    htmlUrl: 'https://github.com/mindwork64/forgedesk/pull/214#pullrequestreview-801',
  },
  {
    id: 802,
    user: { login: 'review-bot' },
    state: 'COMMENTED',
    body: 'Please add a screenshot for the empty repository state.',
    submittedAt: iso(6),
    htmlUrl: null,
  },
]

const checks: GitHubCheckRun[] = [
  { id: 901, name: 'CI (ubuntu-latest)', status: 'completed', conclusion: 'success', startedAt: iso(4), completedAt: iso(3.9), htmlUrl: null, detailsUrl: null },
  { id: 902, name: 'CI (windows-latest)', status: 'completed', conclusion: 'success', startedAt: iso(4), completedAt: iso(3.8), htmlUrl: null, detailsUrl: null },
  { id: 903, name: 'UI visual gates', status: 'completed', conclusion: 'failure', startedAt: iso(4), completedAt: iso(3.7), htmlUrl: null, detailsUrl: null },
  { id: 904, name: 'Tauri shell', status: 'in_progress', conclusion: null, startedAt: iso(1), completedAt: null, htmlUrl: null, detailsUrl: null },
]

const files: GitHubFile[] = [
  {
    filename: 'crates/forgedesk-core/src/github_auth.rs',
    status: 'modified',
    additions: 42,
    deletions: 4,
    changes: 46,
    previousFilename: null,
    patch: '@@ -180,6 +180,18 @@ impl GitHubAuth {\n+    pub fn access_token(&self) -> CoreResult<String> {\n+        self.hydrate_from_keyring()?;\n+        let needs_refresh = ...\n+    }\n@@ -210,4 +222,5 @@\n-        inner.token.as_ref().map(|token| token.access_token.clone())\n+        inner.token.as_ref().map(|token| token.access_token.clone())',
  },
  {
    filename: 'src/features/pull-requests/PullRequestsView.vue',
    status: 'added',
    additions: 260,
    deletions: 0,
    changes: 260,
    previousFilename: null,
    patch: '@@ -0,0 +1,12 @@\n+<script setup lang="ts">\n+import { GitPullRequest } from \'@lucide/vue\'\n+</script>\n+<template>\n+  <div class="fd-pr-page">…</div>\n+</template>',
  },
]

const reviewComments: GitHubReviewComment[] = [
  {
    id: 701,
    body: 'Please keep this aligned with the token vault error category.',
    path: 'crates/forgedesk-core/src/github_auth.rs',
    line: 184,
    originalLine: 180,
    side: 'RIGHT',
    user: { login: 'mindwork64' },
    createdAt: iso(3),
    htmlUrl: 'https://github.com/mindwork64/forgedesk/pull/214#discussion_r701',
  },
  {
    id: 702,
    body: 'The warning should stay non-blocking while patches remain visible.',
    path: 'src/features/pull-requests/PullRequestsView.vue',
    line: 250,
    originalLine: 246,
    side: 'RIGHT',
    user: { login: 'review-bot' },
    createdAt: iso(5),
    htmlUrl: null,
  },
]

const detail: PullRequestDetail = {
  pull: pullOne,
  commits,
  reviews,
  checks,
  files,
  comments: reviewComments,
}

const overLimitFiles: GitHubFile[] = Array.from({ length: 3000 }, (_, index) => ({
  filename: `src/generated/file-${index + 1}.ts`,
  status: 'modified',
  additions: 1,
  deletions: 0,
  changes: 1,
  patch: null,
  previousFilename: null,
}))

const reviewRequests: GitHubSearchIssue[] = [pullOne, pullTwo].map((pull) => ({
  id: pull.id,
  number: pull.number,
  title: pull.title,
  state: pull.state,
  htmlUrl: pull.htmlUrl,
  updatedAt: pull.updatedAt,
  user: pull.user,
}))

const ciOverview: CiOverview = {
  defaultBranch: 'main',
  commits,
  checks,
}

const linkedPull: PullLink = {
  id: 'pull-link-1',
  localIssueId: 'issue-101',
  issueKey: 'FD-142',
  issueTitle: 'GitHub pull request workspace',
  provider: 'github',
  accountId: 'github:34349377',
  repositoryId: 'mindwork64/forgedesk',
  pullNumber: 214,
  pullUrl: pullOne.htmlUrl,
  title: pullOne.title,
  state: 'open',
  createdAt: iso(24),
  updatedAt: iso(2),
}

const capabilities: PullCapabilities = { canManage: true, canReview: true }

const mergeResult: GitHubMergeResult = {
  merged: true,
  message: 'Pull request successfully merged',
  sha: 'f1e2d3c4b5a6978877665544332211aabbccddee',
  completedIssueKey: 'FD-142',
}

const submittedReview: GitHubReview = {
  id: 990,
  user: { login: 'eloraam' },
  state: 'APPROVED',
  body: 'Approved from ForgeDesk.',
  submittedAt: iso(1),
  htmlUrl: null,
}

const overLimitDetail: PullRequestDetail = {
  pull: { ...pullTwo, changedFiles: 3500 },
  commits: [],
  reviews: [],
  checks: [],
  files: overLimitFiles,
  comments: [],
}

export function handleStage6FixtureCommand<T>(
  command: CommandName,
  args?: Record<string, unknown>,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'github_list_pulls':
      return { handled: true, value: [pullOne, pullTwo] as T }
    case 'github_pull_detail':
      return {
        handled: true,
        value: structuredClone(
          Number(args?.number) === pullTwo.number ? overLimitDetail : detail,
        ) as T,
      }
    case 'github_review_requested':
      return { handled: true, value: structuredClone(reviewRequests) as T }
    case 'github_ci_overview':
      return { handled: true, value: structuredClone(ciOverview) as T }
    case 'github_commit_checks':
      return { handled: true, value: structuredClone(checks) as T }
    case 'github_pull_link':
      return { handled: true, value: structuredClone(linkedPull) as T }
    case 'github_link_pull':
      return { handled: true, value: structuredClone(linkedPull) as T }
    case 'github_unlink_pull':
      return { handled: true, value: undefined as T }
    case 'github_pull_capabilities':
      return { handled: true, value: structuredClone(capabilities) as T }
    case 'github_merge_pull':
      return { handled: true, value: structuredClone(mergeResult) as T }
    case 'github_close_pull':
      return {
        handled: true,
        value: structuredClone({ ...pullOne, state: 'closed' }) as T,
      }
    case 'github_submit_review':
      return { handled: true, value: structuredClone(submittedReview) as T }
    case 'github_sync_pull_events':
      return { handled: true, value: 0 as T }
    default:
      return { handled: false }
  }
}

import type {
  CommandName,
  GitHubCheckRun,
  GitHubCommit,
  GitHubPull,
  GitHubReview,
  GitHubFile,
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

const detail: PullRequestDetail = {
  pull: pullOne,
  commits,
  reviews,
  checks,
  files,
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

const overLimitDetail: PullRequestDetail = {
  pull: { ...pullTwo, changedFiles: 3500 },
  commits: [],
  reviews: [],
  checks: [],
  files: overLimitFiles,
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
    default:
      return { handled: false }
  }
}

export const GITHUB_PULL_FILES_LIMIT = 3000

export type FileCompletenessStatus = 'complete' | 'incomplete' | 'unknown'
export type FileCompletenessReason = 'complete' | 'incomplete' | 'over-limit' | 'unknown-cap'

export interface FileCompleteness {
  status: FileCompletenessStatus
  reason: FileCompletenessReason
  loaded: number
  expected: number | null
  remaining: number | null
  excess: number | null
  limit: number
}

/**
 * GitHub's PR files endpoint caps at 3000 files. It also reports changed_files,
 * which may be larger than what the endpoint can return. Never present the
 * capped subset as complete.
 */
export function assessFileCompleteness(
  loaded: number,
  expected: number | null,
): FileCompleteness {
  if (expected !== null) {
    if (expected > loaded) {
      const excess = expected > GITHUB_PULL_FILES_LIMIT
        ? expected - GITHUB_PULL_FILES_LIMIT
        : null
      return {
        status: 'incomplete',
        reason: excess === null ? 'incomplete' : 'over-limit',
        loaded,
        expected,
        remaining: expected - loaded,
        excess,
        limit: GITHUB_PULL_FILES_LIMIT,
      }
    }
    return {
      status: 'complete',
      reason: 'complete',
      loaded,
      expected,
      remaining: 0,
      excess: null,
      limit: GITHUB_PULL_FILES_LIMIT,
    }
  }
  if (loaded >= GITHUB_PULL_FILES_LIMIT) {
    return {
      status: 'unknown',
      reason: 'unknown-cap',
      loaded,
      expected: null,
      remaining: null,
      excess: null,
      limit: GITHUB_PULL_FILES_LIMIT,
    }
  }
  return {
    status: 'complete',
    reason: 'complete',
    loaded,
    expected: null,
    remaining: null,
    excess: null,
    limit: GITHUB_PULL_FILES_LIMIT,
  }
}

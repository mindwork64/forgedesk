export const GITHUB_PULL_FILES_LIMIT = 3000

export type FileCompletenessStatus = 'complete' | 'incomplete' | 'unknown'

export interface FileCompleteness {
  status: FileCompletenessStatus
  loaded: number
  expected: number | null
  remaining: number | null
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
      return {
        status: 'incomplete',
        loaded,
        expected,
        remaining: expected - loaded,
        limit: GITHUB_PULL_FILES_LIMIT,
      }
    }
    return {
      status: 'complete',
      loaded,
      expected,
      remaining: 0,
      limit: GITHUB_PULL_FILES_LIMIT,
    }
  }
  if (loaded >= GITHUB_PULL_FILES_LIMIT) {
    return {
      status: 'unknown',
      loaded,
      expected: null,
      remaining: null,
      limit: GITHUB_PULL_FILES_LIMIT,
    }
  }
  return {
    status: 'complete',
    loaded,
    expected: null,
    remaining: null,
    limit: GITHUB_PULL_FILES_LIMIT,
  }
}

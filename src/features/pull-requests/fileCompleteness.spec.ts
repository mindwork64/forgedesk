import { describe, expect, it } from 'vitest'

import { assessFileCompleteness, GITHUB_PULL_FILES_LIMIT } from './fileCompleteness'

describe('PR file completeness', () => {
  it('treats 100/100 as complete', () => {
    expect(assessFileCompleteness(100, 100)).toMatchObject({ status: 'complete', reason: 'complete' })
  })

  it('treats 2999/2999 as complete', () => {
    expect(assessFileCompleteness(2999, 2999)).toMatchObject({ status: 'complete', reason: 'complete' })
  })

  it('treats 3000/3000 as complete despite the API limit', () => {
    expect(assessFileCompleteness(3000, 3000)).toMatchObject({ status: 'complete', reason: 'complete' })
  })

  it('marks 3000/3500 incomplete with the missing count', () => {
    const result = assessFileCompleteness(3000, 3500)
    expect(result.status).toBe('incomplete')
    expect(result.reason).toBe('over-limit')
    expect(result.remaining).toBe(500)
    expect(result.excess).toBe(500)
    expect(result.limit).toBe(GITHUB_PULL_FILES_LIMIT)
  })

  it('marks 3000/unknown as indeterminate', () => {
    expect(assessFileCompleteness(3000, null)).toMatchObject({ status: 'unknown', reason: 'unknown-cap' })
  })

  it('detects a non-cap incomplete list (2/9)', () => {
    expect(assessFileCompleteness(2, 9)).toMatchObject({
      status: 'incomplete',
      reason: 'incomplete',
      remaining: 7,
      excess: null,
    })
  })

  it('detects expected > actual below 3000', () => {
    const result = assessFileCompleteness(2999, 3000)
    expect(result.status).toBe('incomplete')
    expect(result.reason).toBe('incomplete')
    expect(result.remaining).toBe(1)
    expect(result.excess).toBeNull()
  })

  it('does not mark 2999/unknown incomplete', () => {
    expect(assessFileCompleteness(2999, null).status).toBe('complete')
  })
})

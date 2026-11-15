import { describe, expect, it } from 'vitest'

import { assessFileCompleteness, GITHUB_PULL_FILES_LIMIT } from './fileCompleteness'

describe('PR file completeness', () => {
  it('treats 100/100 as complete', () => {
    expect(assessFileCompleteness(100, 100).status).toBe('complete')
  })

  it('treats 2999/2999 as complete', () => {
    expect(assessFileCompleteness(2999, 2999).status).toBe('complete')
  })

  it('treats 3000/3000 as complete despite the API limit', () => {
    expect(assessFileCompleteness(3000, 3000).status).toBe('complete')
  })

  it('marks 3000/3500 incomplete with the missing count', () => {
    const result = assessFileCompleteness(3000, 3500)
    expect(result.status).toBe('incomplete')
    expect(result.remaining).toBe(500)
    expect(result.limit).toBe(GITHUB_PULL_FILES_LIMIT)
  })

  it('marks 3000/unknown as indeterminate', () => {
    expect(assessFileCompleteness(3000, null).status).toBe('unknown')
  })

  it('detects expected > actual below 3000', () => {
    const result = assessFileCompleteness(2999, 3000)
    expect(result.status).toBe('incomplete')
    expect(result.remaining).toBe(1)
  })

  it('does not mark 2999/unknown incomplete', () => {
    expect(assessFileCompleteness(2999, null).status).toBe('complete')
  })
})

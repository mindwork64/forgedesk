import { describe, expect, it } from 'vitest'

import { normaliseRoute } from './routes'

describe('normaliseRoute', () => {
  it('keeps known internal routes', () => {
    expect(normaliseRoute('/my-issues')).toBe('/my-issues')
    expect(normaliseRoute('/settings/storage')).toBe('/settings/storage')
    expect(normaliseRoute('/api-monitor')).toBe('/api-monitor')
  })

  it('rejects external, protocol-relative and traversal values', () => {
    expect(normaliseRoute('https://evil.example')).toBe('/my-issues')
    expect(normaliseRoute('//evil.example')).toBe('/my-issues')
    expect(normaliseRoute('/../../etc/passwd')).toBe('/my-issues')
    expect(normaliseRoute('/unknown-page')).toBe('/my-issues')
    expect(normaliseRoute('')).toBe('/my-issues')
  })

  it('falls back to My Issues for the root route', () => {
    expect(normaliseRoute('/')).toBe('/my-issues')
  })

  it('drops query strings before matching', () => {
    expect(normaliseRoute('/projects?view=board')).toBe('/projects')
  })
})

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const css = readFileSync(resolve(process.cwd(), 'src/styles/tokens.css'), 'utf8')

const DARK_TOKENS: Record<string, string> = {
  '--fd-bg': '#0e1117',
  '--fd-sidebar': '#11151c',
  '--fd-surface': '#151a22',
  '--fd-surface-raised': '#1a202b',
  '--fd-surface-hover': '#202735',
  '--fd-surface-selected': '#242d3d',
  '--fd-border': '#293241',
  '--fd-border-subtle': '#202734',
  '--fd-text': '#e8ecf2',
  '--fd-text-secondary': '#a0a9b8',
  '--fd-text-muted': '#727d8d',
  '--fd-accent': '#8b7cff',
  '--fd-accent-soft': '#302b52',
  '--fd-success': '#43b581',
  '--fd-warning': '#e5a84b',
  '--fd-danger': '#e16d75',
  '--fd-info': '#6fa8ff',
}

const LIGHT_TOKENS: Record<string, string> = {
  '--fd-bg': '#f5f6f8',
  '--fd-sidebar': '#eceff3',
  '--fd-surface': '#ffffff',
  '--fd-surface-raised': '#ffffff',
  '--fd-surface-hover': '#eef1f5',
  '--fd-surface-selected': '#e7e9f8',
  '--fd-border': '#d8dde5',
  '--fd-border-subtle': '#e7eaf0',
  '--fd-text': '#202633',
  '--fd-text-secondary': '#5f6978',
  '--fd-text-muted': '#8a93a1',
  '--fd-accent': '#6757d9',
  '--fd-accent-soft': '#eae7ff',
  '--fd-success': '#23865b',
  '--fd-warning': '#a86a12',
  '--fd-danger': '#c44450',
  '--fd-info': '#356fc4',
}

const GEOMETRY: Record<string, string> = {
  '--fd-sidebar-width': '240px',
  '--fd-sidebar-collapsed-width': '64px',
  '--fd-header-height': '48px',
  '--fd-status-height': '24px',
  '--fd-detail-width': '440px',
  '--fd-content-pad': '24px',
  '--fd-sidebar-pad': '12px',
  '--fd-nav-item-height': '32px',
  '--fd-row-height': '40px',
  '--fd-row-height-compact': '36px',
  '--fd-button-height': '32px',
  '--fd-button-height-sm': '28px',
  '--fd-input-height': '32px',
  '--fd-toolbar-height': '40px',
  '--fd-radius-control': '6px',
  '--fd-radius-panel': '8px',
  '--fd-radius-modal': '10px',
}

describe('design tokens contract', () => {
  it.each(Object.entries(DARK_TOKENS))('dark %s matches 06_UI_UX_SPEC', (token, value) => {
    expect(css).toContain(`${token}: ${value};`)
  })

  it.each(Object.entries(LIGHT_TOKENS))('light %s matches 06_UI_UX_SPEC', (token, value) => {
    expect(css).toContain(`${token}: ${value};`)
  })

  it.each(Object.entries(GEOMETRY))('%s matches shell geometry', (token, value) => {
    expect(css).toContain(`${token}: ${value};`)
  })

  it('defines every base text size from 11 to 20 px', () => {
    for (const size of [11, 12, 13, 14, 16, 18, 20]) {
      expect(css).toContain(`--fd-text-${size}: ${size}px;`)
    }
  })
})

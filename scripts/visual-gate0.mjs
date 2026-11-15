/**
 * UI Gate 0 visual acceptance script.
 *
 * Prerequisites:
 *   VITE_VISUAL_HARNESS=true npm run build
 *   npm run preview -- --host 127.0.0.1 --port 4173
 *
 * Run:
 *   PLAYWRIGHT_BROWSERS_PATH=.playwright-browsers node scripts/visual-gate0.mjs
 *
 * The harness is a documented test fixture (see docs/ui/visual_baseline.md); it
 * is not part of a production build. Native-shell screenshots require WebKitGTK
 * and are captured separately on a packaging machine.
 */
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { chromium } from 'playwright'

const BASE_URL = process.env.PREVIEW_URL ?? 'http://127.0.0.1:4173'
const OUT_DIR = resolve('docs/ui/current')

const cases = [
  {
    name: 'gate0-shell-dark-1440-expanded',
    waitFor: '[data-testid="issue-row"]',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-light-1440-expanded',
    waitFor: '[data-testid="issue-row"]',
    theme: 'light',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1280-collapsed',
    waitFor: '[data-testid="issue-row"]',
    theme: 'dark',
    sidebar: 'collapsed',
    width: 1280,
    height: 800,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-light-1280-collapsed',
    waitFor: '[data-testid="issue-row"]',
    theme: 'light',
    sidebar: 'collapsed',
    width: 1280,
    height: 800,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1100-collapsed',
    waitFor: '[data-testid="issue-row"]',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1100,
    height: 700,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1440-russian',
    waitFor: '[data-testid="issue-row"]',
    theme: 'dark',
    locale: 'ru',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
  },
  {
    name: 'gate0-settings-storage-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/settings/storage',
    waitFor: '[data-testid="settings-content"]',
  },
  {
    name: 'gate0-settings-storage-light-1440',
    theme: 'light',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/settings/storage',
    waitFor: '[data-testid="settings-content"]',
  },
  {
    name: 'gate0-restore-dialog-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/settings/storage',
    interaction: 'restore-dialog',
    waitFor: '.fd-modal-panel',
  },
  {
    name: 'gate0-onboarding-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    onboarding: true,
    waitFor: '.fd-onboarding',
  },
  {
    name: 'gate0-command-palette-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    palette: true,
    waitFor: '.fd-palette',
  },
  {
    name: 'gate0-status-popover-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    interaction: 'status-popover',
    waitFor: '.fd-popover__panel',
  },
  {
    name: 'gate0-tooltip-dark-1280',
    theme: 'dark',
    sidebar: 'collapsed',
    width: 1280,
    height: 800,
    route: '/my-issues',
    interaction: 'tooltip',
  },
  {
    name: 'gate0-toast-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    interaction: 'toast',
    waitFor: '.fd-toast',
  },
  {
    name: 'gate0-inbox-shortcut-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    interaction: 'chord',
  },
  {
    name: 'gate1-my-issues-list-detail-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    selectIssue: 'FD-101',
    waitFor: '[data-testid="issue-detail-title"]',
  },
  {
    name: 'gate1-detail-overlay-1100',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1100,
    height: 700,
    route: '/my-issues',
    selectIssue: 'FD-101',
    waitFor: '[data-testid="issue-detail-title"]',
  },
  {
    name: 'gate1-my-issues-list-light-1440',
    theme: 'light',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    waitFor: '[data-testid="issue-row"]',
  },
  {
    name: 'gate1-my-issues-board-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    view: 'board',
    waitFor: '.fd-board',
  },
  {
    name: 'gate1-create-dialog-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    openCreate: true,
    waitFor: '.fd-issue-create',
  },
  {
    name: 'gate1-detail-activity-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    selectIssue: 'FD-101',
    scrollDetail: true,
    waitFor: '[data-testid="issue-detail-title"]',
  },
  {
    name: 'gate1-filter-builder-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    openFilter: true,
    addFilterCondition: true,
    waitFor: '.fd-filter-row',
  },
  {
    name: 'gate1-filter-applied-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    openFilter: true,
    addFilterCondition: true,
    applyFilter: true,
    waitFor: '.fd-filter-chip',
  },
  {
    name: 'gate1-grouped-list-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    groupBy: 'status',
    waitFor: '.fd-issue-group__head',
  },
  {
    name: 'gate1-active-view-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    savedView: 'Active work',
    waitFor: '.fd-issue-group__head',
  },
  {
    name: 'gate1-saved-views-menu-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    openSavedViews: true,
    waitFor: '.fd-saved-views',
  },
  {
    name: 'gate1-save-view-dialog-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    openSavedViews: true,
    openSaveView: true,
    waitFor: '.fd-save-view',
  },
  {
    name: 'gate1-palette-issues-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    paletteQuery: 'capture',
    waitFor: '.fd-palette',
  },
  {
    name: 'gate2-projects-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/projects',
    waitFor: '.fd-project-row',
  },
  {
    name: 'gate2-project-dialog-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/projects',
    openProjectDialog: true,
    waitFor: '.fd-project-form',
  },
  {
    name: 'gate2-project-overview-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/projects/project-exhi',
    waitFor: '.fd-project-tabs',
  },
  {
    name: 'gate2-project-issues-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/projects/project-exhi',
    projectTab: 'Issues',
    waitFor: '.fd-issue-list',
  },
  {
    name: 'gate2-initiatives-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/initiatives',
    waitFor: '.fd-initiative-row',
  },
  {
    name: 'gate2-initiative-overview-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/initiatives/init-capture',
    waitFor: '.fd-initiative-section',
  },
  {
    name: 'gate2-cycles-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/cycles',
    waitFor: '.fd-cycle-summary',
  },
  {
    name: 'gate2-documents-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/documents',
    waitFor: '.fd-document-editor',
  },
  {
    name: 'gate2-roadmap-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/roadmap',
    waitFor: '.fd-roadmap__bar',
  },
  {
    name: 'gate3-repositories-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/repositories',
    waitFor: '.fd-repository-row',
  },
  {
    name: 'gate3-repository-detail-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/repositories/repo-forge',
    waitFor: '.fd-repository-tabs',
  },
  {
    name: 'gate3-repository-dirty-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/repositories/repo-exhi',
    waitFor: '.fd-repository-status--dirty',
  },
  {
    name: 'gate3-integrations-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/integrations',
    waitFor: '.fd-provider-card',
  },
  {
    name: 'gate3-github-auth-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/integrations',
    openGithubFlow: true,
    waitFor: '.fd-github-device',
  },
  {
    name: 'gate3-github-repos-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/integrations',
    openGithubRepos: true,
    waitFor: '.fd-github-repos',
  },
  {
    name: 'gate3-prs-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    waitFor: '.fd-pr-table',
  },
  {
    name: 'gate3-pr-detail-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    openPullRequest: true,
    waitFor: '[data-testid="detail-pane"]',
  },
  {
    name: 'gate3-pr-files-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    openPullRequest: true,
    openPullFiles: true,
    waitFor: '[data-testid="detail-pane"]',
  },
  {
    name: 'gate3-pr-merge-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    openPullRequest: true,
    openMergeDialog: true,
    waitFor: '[data-testid="detail-pane"]',
  },
  {
    name: 'gate3-pr-files-split-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    openPullRequest: true,
    openPullFiles: true,
    splitDiff: true,
    waitFor: '[data-testid="detail-pane"]',
  },
  {
    name: 'gate3-pr-files-overlimit-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/pull-requests',
    openPullRequest: true,
    openPullRow: 1,
    openPullFiles: true,
    waitFor: '[data-testid="detail-pane"]',
  },
  {
    name: 'gate3-reviews-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/reviews',
    waitFor: '.fd-reviews-table',
  },
  {
    name: 'gate3-ci-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/ci',
    waitFor: '.fd-ci-check-list',
  },
  {
    name: 'gate4-inbox-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/inbox',
    waitFor: '.fd-inbox-row',
  },
  {
    name: 'gate4-insights-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/insights',
    waitFor: '.fd-insight-card',
  },
  {
    name: 'gate4-automations-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/automations',
    waitFor: '.fd-auto-row',
  },
  {
    name: 'gate3-api-monitor-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/api-monitor',
    waitFor: '.fd-budget-panel',
  },
  {
    name: 'gate0-startup-error-dark-1440',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    error: true,
    waitFor: '[data-testid="startup-error"]',
  },
]

function buildUrl(testCase) {
  const query = new URLSearchParams({
    theme: testCase.theme,
    sidebar: testCase.sidebar,
    route: testCase.route,
  })
  if (testCase.onboarding) {
    query.set('onboarding', '1')
  }
  if (testCase.error) {
    query.set('error', '1')
  }
  if (testCase.locale) {
    query.set('locale', testCase.locale)
  }
  return `${BASE_URL}/?${query.toString()}#${testCase.route}`
}

async function measure(page) {
  return page.evaluate(() => {
    const rect = (selector) => {
      const element = document.querySelector(selector)
      if (!element) {
        return null
      }
      const bounds = element.getBoundingClientRect()
      return {
        x: Math.round(bounds.x),
        y: Math.round(bounds.y),
        width: Math.round(bounds.width),
        height: Math.round(bounds.height),
      }
    }
    const pageHeader = document.querySelector('[data-testid="page-header"]')
    const sidebar = document.querySelector('[data-testid="app-sidebar"]')
    const tokens = getComputedStyle(document.documentElement)
    return {
      viewport: { width: window.innerWidth, height: window.innerHeight },
      sidebar: rect('[data-testid="app-sidebar"]'),
      header: rect('[data-testid="app-header"]'),
      status: rect('[data-testid="status-strip"]'),
      pageHeader: rect('[data-testid="page-header"]'),
      contentPaddingLeft: pageHeader
        ? getComputedStyle(pageHeader).paddingLeft
        : null,
      bodyBackground: getComputedStyle(document.body).backgroundColor,
      sidebarBackground: sidebar
        ? getComputedStyle(sidebar).backgroundColor
        : null,
      fontFamily: getComputedStyle(document.body).fontFamily,
      fontSize: getComputedStyle(document.body).fontSize,
      overflowX:
        Math.max(
          document.documentElement.scrollWidth,
          document.body.scrollWidth,
        ) > window.innerWidth,
      activeNavItems: document.querySelectorAll(
        '[data-testid="app-sidebar"] [aria-current="page"]',
      ).length,
      tokenValues: {
        sidebarWidth: tokens.getPropertyValue('--fd-sidebar-width').trim(),
        collapsedWidth: tokens
          .getPropertyValue('--fd-sidebar-collapsed-width')
          .trim(),
        headerHeight: tokens.getPropertyValue('--fd-header-height').trim(),
        statusHeight: tokens.getPropertyValue('--fd-status-height').trim(),
        contentPadding: tokens.getPropertyValue('--fd-content-pad').trim(),
      },
    }
  })
}

function assert(condition, message, failures) {
  if (!condition) {
    failures.push(message)
  }
}

function checkGeometry(name, metrics, expected, failures) {
  const { sidebar, header, status } = metrics
  assert(sidebar !== null, `${name}: sidebar missing`, failures)
  assert(header !== null, `${name}: header missing`, failures)
  assert(status !== null, `${name}: status strip missing`, failures)
  if (!sidebar || !header || !status) {
    return
  }
  assert(
    Math.abs(sidebar.width - expected.sidebarWidth) <= 2,
    `${name}: sidebar width ${sidebar.width}, expected ${expected.sidebarWidth}`,
    failures,
  )
  assert(
    Math.abs(header.height - 48) <= 2,
    `${name}: header height ${header.height}, expected 48`,
    failures,
  )
  assert(
    Math.abs(status.height - 24) <= 2,
    `${name}: status height ${status.height}, expected 24`,
    failures,
  )
  assert(
    metrics.contentPaddingLeft === '24px',
    `${name}: content padding ${metrics.contentPaddingLeft}, expected 24px`,
    failures,
  )
  assert(
    metrics.bodyBackground === expected.bodyBackground,
    `${name}: body background ${metrics.bodyBackground}, expected ${expected.bodyBackground}`,
    failures,
  )
  assert(
    metrics.sidebarBackground === expected.sidebarBackground,
    `${name}: sidebar background ${metrics.sidebarBackground}, expected ${expected.sidebarBackground}`,
    failures,
  )
  assert(!metrics.overflowX, `${name}: horizontal overflow detected`, failures)
  assert(
    metrics.activeNavItems === 1,
    `${name}: expected exactly one active nav item, found ${metrics.activeNavItems}`,
    failures,
  )
  assert(
    metrics.fontSize === '13px',
    `${name}: base font ${metrics.fontSize}, expected 13px`,
    failures,
  )
  assert(
    metrics.tokenValues.sidebarWidth === '240px' &&
      metrics.tokenValues.collapsedWidth === '64px' &&
      metrics.tokenValues.headerHeight === '48px' &&
      metrics.tokenValues.statusHeight === '24px' &&
      metrics.tokenValues.contentPadding === '24px',
    `${name}: geometry tokens differ from the contract`,
    failures,
  )
}

async function main() {
  mkdirSync(OUT_DIR, { recursive: true })

  const browser = await chromium.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage'],
  })

  const report = {
    generatedAt: new Date().toISOString(),
    baseUrl: BASE_URL,
    harness: true,
    cases: [],
    assertions: [],
    failures: [],
    screenshots: [],
  }

  try {
    for (const testCase of cases) {
      const context = await browser.newContext({
        viewport: { width: testCase.width, height: testCase.height },
        deviceScaleFactor: 1,
        colorScheme: testCase.theme,
        locale: 'en-US',
      })
      const page = await context.newPage()
      const consoleErrors = []
      page.on('console', (message) => {
        if (message.type() === 'error') {
          consoleErrors.push(message.text())
        }
      })
      page.on('pageerror', (error) => consoleErrors.push(String(error)))

      await page.goto(buildUrl(testCase), { waitUntil: 'load' })
      await page.waitForSelector(
        testCase.error
          ? '[data-testid="startup-error"]'
          : '[data-testid="app-shell"]',
        { timeout: 10_000 },
      )

      if (!testCase.error && testCase.theme) {
        await page.waitForFunction(
          (theme) => document.documentElement.dataset.theme === theme,
          testCase.theme,
        )
      }

      if (testCase.palette) {
        await page.keyboard.press('Control+k')
      }

      if (testCase.interaction === 'status-popover') {
        await page.click('.fd-status-strip__state')
      }

      if (testCase.interaction === 'tooltip') {
        await page.hover('[data-testid="app-sidebar"] a[aria-label="Projects"]')
        await page.waitForFunction(() => {
          const tooltip = document.querySelector('.fd-tooltip')
          return tooltip && getComputedStyle(tooltip).visibility === 'visible'
        })
      }

      if (testCase.interaction === 'toast') {
        await page.keyboard.press('Control+k')
        await page.waitForSelector('.fd-palette')
        await page.fill('.fd-palette__input', 'backup')
        await page.keyboard.press('Enter')
      }

      if (testCase.interaction === 'chord') {
        await page.keyboard.press('g')
        await page.keyboard.press('i')
        await page.waitForFunction(() => window.location.hash === '#/inbox')
      }

      if (testCase.view === 'board') {
        await page.getByRole('radio', { name: 'Board' }).click()
        await page.waitForSelector('.fd-board')
      }

      if (testCase.selectIssue) {
        await page
          .locator('[data-testid="issue-row"]')
          .filter({ hasText: testCase.selectIssue })
          .first()
          .click()
        await page.waitForSelector('[data-testid="issue-detail-title"]')
      }

      if (testCase.openFilter) {
        await page.getByRole('button', { name: /^Filter/ }).first().click()
        await page.waitForSelector('.fd-filter-builder')
      }

      if (testCase.addFilterCondition) {
        await page.getByRole('button', { name: 'Add condition' }).click()
        await page.waitForSelector('.fd-filter-row')
      }

      if (testCase.applyFilter) {
        await page.getByRole('button', { name: 'Apply filters' }).click()
      }

      if (testCase.groupBy) {
        await page.getByRole('button', { name: /^Group/ }).first().click()
        await page.waitForSelector('.fd-group-menu')
        await page.locator('.fd-group-menu select').selectOption(testCase.groupBy)
        await page.getByRole('button', { name: 'Apply', exact: true }).click()
        await page.waitForSelector('.fd-issue-group__head')
      }

      if (testCase.savedView) {
        await page
          .getByRole('button', { name: 'Saved view', exact: true })
          .first()
          .click()
        await page.waitForSelector('.fd-saved-views')
        await page
          .getByRole('button', { name: testCase.savedView, exact: true })
          .click()
        await page.waitForTimeout(400)
      }

      if (testCase.openGithubFlow) {
        await page.getByRole('button', { name: 'Reauthorize' }).click()
        await page.waitForSelector('.fd-github-device')
        await page.waitForTimeout(250)
      }

      if (testCase.openPullRequest) {
        await page.locator('.fd-pr-row').nth(testCase.openPullRow ?? 0).click()
        await page.waitForSelector('[data-testid="detail-pane"]')
        await page.waitForTimeout(300)
      }

      if (testCase.openMergeDialog) {
        await page.getByRole('button', { name: 'Merge', exact: true }).click()
        await page.waitForSelector('.fd-merge')
        await page.waitForTimeout(200)
      }

      if (testCase.openPullFiles) {
        await page.getByRole('button', { name: 'Files', exact: true }).click()
        await page.waitForSelector('.fd-pr-file')
        await page.waitForTimeout(200)
      }

      if (testCase.splitDiff) {
        await page.getByRole('radio', { name: 'Split', exact: true }).click()
        await page.waitForSelector('.fd-pr-split__row')
        await page.waitForTimeout(200)
      }

      if (testCase.openGithubRepos) {
        await page.getByRole('button', { name: 'Discover repositories' }).click()
        await page.waitForSelector('.fd-github-repos')
        await page.waitForTimeout(250)
      }

      if (testCase.openSavedViews) {
        await page
          .getByRole('button', { name: 'Saved view', exact: true })
          .first()
          .click()
        await page.waitForSelector('.fd-saved-views')
      }

      if (testCase.openSaveView) {
        await page.getByRole('button', { name: 'Save view', exact: true }).click()
        await page.waitForSelector('.fd-save-view')
      }

      if (testCase.openProjectDialog) {
        await page.getByRole('button', { name: 'New project', exact: true }).click()
        await page.waitForSelector('.fd-project-form')
      }

      if (testCase.projectTab) {
        await page
          .locator('.fd-project-tab')
          .filter({ hasText: testCase.projectTab })
          .click()
        await page.waitForSelector('.fd-issue-list')
      }

      if (testCase.paletteQuery) {
        await page.keyboard.press('Control+k')
        await page.waitForSelector('.fd-palette')
        await page.fill('.fd-palette__input', testCase.paletteQuery)
        await page.waitForTimeout(450)
      }

      if (testCase.openCreate) {
        await page.getByRole('button', { name: 'New issue' }).click()
        await page.waitForSelector('.fd-issue-create')
      }

      if (testCase.scrollDetail) {
        await page.evaluate(() => {
          const body = document.querySelector('.fd-detail-body')
          if (body) body.scrollTop = body.scrollHeight
        })
      }

      if (testCase.interaction === 'restore-dialog') {
        await page
          .getByRole('button', { name: 'Restore', exact: true })
          .first()
          .click()
      }

      if (testCase.waitFor) {
        await page.waitForSelector(testCase.waitFor, { timeout: 10_000 })
      }

      await page.waitForTimeout(250)

      const screenshot = `${testCase.name}.png`
      await page.screenshot({
        path: resolve(OUT_DIR, screenshot),
        animations: 'disabled',
      })

      const caseReport = {
        name: testCase.name,
        viewport: `${testCase.width}x${testCase.height}`,
        theme: testCase.theme,
        route: testCase.route,
        screenshot: `docs/ui/current/${screenshot}`,
        consoleErrors,
      }

      if (testCase.measure) {
        const metrics = await measure(page)
        caseReport.metrics = metrics
        const expected = {
          sidebarWidth:
            testCase.sidebar === 'collapsed' || testCase.width < 1200
              ? 64
              : 240,
          bodyBackground:
            testCase.theme === 'dark'
              ? 'rgb(14, 17, 23)'
              : 'rgb(245, 246, 248)',
          sidebarBackground:
            testCase.theme === 'dark'
              ? 'rgb(17, 21, 28)'
              : 'rgb(236, 239, 243)',
        }
        checkGeometry(testCase.name, metrics, expected, report.failures)

        if (testCase.width >= 1280 && testCase.height >= 800) {
          const apiMonitorVisible = await page.evaluate(() => {
            const link = Array.from(
              document.querySelectorAll(
                '[data-testid="app-sidebar"] a[aria-label="API Monitor"]',
              ),
            )[0]
            const sidebar = document.querySelector('[data-testid="app-sidebar"]')
            if (!link || !sidebar) {
              // Expanded mode uses the visible text rather than aria-label.
              const byText = Array.from(
                document.querySelectorAll('[data-testid="app-sidebar"] a'),
              ).find((item) => item.textContent?.trim() === 'API Monitor')
              if (!byText || !sidebar) {
                return false
              }
              const rect = byText.getBoundingClientRect()
              const bounds = sidebar.getBoundingClientRect()
              return rect.top >= bounds.top && rect.bottom <= bounds.bottom
            }
            const rect = link.getBoundingClientRect()
            const bounds = sidebar.getBoundingClientRect()
            return rect.top >= bounds.top && rect.bottom <= bounds.bottom
          })
          assert(
            apiMonitorVisible,
            `${testCase.name}: API Monitor nav item is clipped`,
            report.failures,
          )
        }
      }

      if (testCase.palette) {
        const paletteWidth = await page.evaluate(() => {
          const palette = document.querySelector('.fd-palette')
          return palette
            ? Math.round(palette.getBoundingClientRect().width)
            : null
        })
        caseReport.paletteWidth = paletteWidth
        assert(
          paletteWidth !== null && paletteWidth <= 640 && paletteWidth > 500,
          `${testCase.name}: palette width ${paletteWidth}, expected ≤ 640`,
          report.failures,
        )
      }

      if (testCase.interaction === 'tooltip') {
        const tooltipText = await page.evaluate(() => {
          const tooltip = document.querySelector('.fd-tooltip')
          return tooltip ? tooltip.textContent?.trim() : null
        })
        caseReport.tooltipText = tooltipText
        assert(
          tooltipText === 'Projects',
          `${testCase.name}: tooltip text ${tooltipText}, expected Projects`,
          report.failures,
        )
      }

      if (testCase.interaction === 'toast') {
        const toastText = await page.evaluate(() => {
          const toast = document.querySelector('.fd-toast')
          return toast ? toast.textContent?.trim() : null
        })
        caseReport.toastText = toastText
        assert(
          toastText !== null && toastText.length > 0,
          `${testCase.name}: toast did not appear`,
          report.failures,
        )
      }

      if (testCase.interaction === 'chord') {
        const hash = await page.evaluate(() => window.location.hash)
        caseReport.hash = hash
        assert(
          hash === '#/inbox',
          `${testCase.name}: chord G then I produced ${hash}`,
          report.failures,
        )
      }

      if (testCase.interaction === 'restore-dialog') {
        const dialogText = await page.evaluate(() => {
          const dialog = document.querySelector('.fd-modal-panel')
          return dialog ? dialog.textContent?.trim() : null
        })
        caseReport.dialogText = dialogText
        assert(
          dialogText !== null && dialogText.includes('Restore this backup?'),
          `${testCase.name}: restore confirmation dialog missing`,
          report.failures,
        )
      }

      if (testCase.selectIssue) {
        const pane = await page.evaluate(() => {
          const element = document.querySelector('[data-testid="detail-pane"]')
          if (!element) return null
          const rect = element.getBoundingClientRect()
          return { width: Math.round(rect.width), left: Math.round(rect.left) }
        })
        caseReport.detailPane = pane
        if (testCase.width >= 1280) {
          assert(
            pane !== null && Math.abs(pane.width - 440) <= 2,
            `${testCase.name}: detail pane width ${pane?.width}, expected 440`,
            report.failures,
          )
        }
        if (testCase.width < 1200) {
          const position = await page.evaluate(() => {
            const element = document.querySelector('[data-testid="detail-pane"]')
            return element ? getComputedStyle(element).position : null
          })
          caseReport.detailPanePosition = position
          assert(
            position === 'absolute',
            `${testCase.name}: detail pane should overlay below 1200 px, got ${position}`,
            report.failures,
          )
        }
      }

      if (testCase.view === 'board') {
        const board = await page.evaluate(() => {
          const columns = Array.from(document.querySelectorAll('.fd-board__column'))
          return {
            columns: columns.length,
            cards: columns.slice(0, 4).map((column) => column.querySelectorAll('.fd-board__card').length),
          }
        })
        caseReport.board = board
        assert(board.columns >= 4, `${testCase.name}: expected ≥4 board columns`, report.failures)
        assert(
          board.cards.every((count) => count >= 3),
          `${testCase.name}: expected ≥3 cards in each of the first four columns, got ${board.cards}`,
          report.failures,
        )
      }

      if (testCase.openCreate) {
        const width = await page.evaluate(() => {
          const modal = document.querySelector('.fd-modal-panel')
          return modal ? Math.round(modal.getBoundingClientRect().width) : null
        })
        caseReport.createDialogWidth = width
        assert(
          width !== null && width <= 720 && width >= 600,
          `${testCase.name}: create dialog width ${width}, expected 600–720`,
          report.failures,
        )
      }

      if (testCase.addFilterCondition && !testCase.applyFilter) {
        const rows = await page.locator('.fd-filter-row').count()
        caseReport.filterRows = rows
        assert(rows >= 1, `${testCase.name}: expected a filter condition row`, report.failures)
      }

      if (testCase.applyFilter) {
        const chips = await page.locator('.fd-filter-chip').count()
        const rows = await page.locator('[data-testid="issue-row"]').count()
        caseReport.filterChips = chips
        caseReport.filteredRows = rows
        assert(chips >= 1, `${testCase.name}: expected an active filter chip`, report.failures)
        assert(rows > 0 && rows < 12, `${testCase.name}: filter did not reduce the list`, report.failures)
      }

      if (testCase.groupBy || testCase.savedView) {
        const groups = await page.locator('.fd-issue-group__head').count()
        caseReport.groups = groups
        assert(groups >= 2, `${testCase.name}: expected grouped list sections`, report.failures)
      }

      if (testCase.openSavedViews && !testCase.openSaveView) {
        const count = await page.locator('.fd-saved-views__row').count()
        caseReport.savedViews = count
        assert(count >= 2, `${testCase.name}: expected seeded saved views`, report.failures)
      }

      if (testCase.openSaveView) {
        const visible = await page.locator('.fd-save-view').isVisible()
        assert(visible, `${testCase.name}: save view dialog is not visible`, report.failures)
      }

      if (testCase.paletteQuery) {
        const issues = await page
          .locator('.fd-palette__group')
          .filter({ hasText: 'Issues' })
          .locator('.fd-palette__item')
          .count()
        caseReport.paletteIssueResults = issues
        assert(issues >= 1, `${testCase.name}: palette returned no issue results`, report.failures)
      }

      if (testCase.openProjectDialog) {
        const width = await page.evaluate(() => {
          const modal = document.querySelector('.fd-modal-panel')
          return modal ? Math.round(modal.getBoundingClientRect().width) : null
        })
        caseReport.projectDialogWidth = width
        assert(
          width !== null && width >= 540 && width <= 570,
          `${testCase.name}: project dialog width ${width}, expected 560`,
          report.failures,
        )
      }

      if (testCase.route === '/projects') {
        const rows = await page.locator('.fd-project-row').count()
        caseReport.projectRows = rows
        assert(rows >= 3, `${testCase.name}: expected project rows`, report.failures)
      }

      if (testCase.route === '/projects/project-exhi') {
        const tabs = await page.locator('.fd-project-tab').count()
        caseReport.projectTabs = tabs
        assert(tabs === 6, `${testCase.name}: expected six project tabs`, report.failures)
        if (testCase.projectTab === 'Issues') {
          const rows = await page.locator('[data-testid="issue-row"]').count()
          caseReport.projectIssueRows = rows
          assert(rows >= 3, `${testCase.name}: expected project issues`, report.failures)
        }
      }

      if (testCase.route === '/initiatives') {
        const rows = await page.locator('.fd-initiative-row').count()
        caseReport.initiativeRows = rows
        assert(rows >= 2, `${testCase.name}: expected initiative rows`, report.failures)
      }

      if (testCase.route === '/initiatives/init-capture') {
        const projects = await page.locator('.fd-initiative-project-row').count()
        caseReport.initiativeProjects = projects
        assert(projects >= 2, `${testCase.name}: expected linked initiatives projects`, report.failures)
      }

      if (testCase.route === '/cycles') {
        const metrics = await page.locator('.fd-cycle-metric').count()
        caseReport.cycleMetrics = metrics
        assert(metrics === 5, `${testCase.name}: expected five cycle metrics`, report.failures)
      }

      if (testCase.route === '/documents') {
        const items = await page.locator('.fd-documents-item').count()
        caseReport.documentItems = items
        assert(items >= 3, `${testCase.name}: expected documents`, report.failures)
      }

      if (testCase.route === '/repositories') {
        const rows = await page.locator('.fd-repository-row').count()
        caseReport.repositoryRows = rows
        assert(rows >= 2, `${testCase.name}: expected local repositories`, report.failures)
      }

      if (testCase.route.startsWith('/repositories/')) {
        const branches = await page.locator('.fd-branch-row').count()
        caseReport.repositoryBranches = branches
        assert(branches >= 2, `${testCase.name}: expected branches`, report.failures)
        if (testCase.route.endsWith('repo-exhi')) {
          const dirty = await page.locator('.fd-repository-status--dirty').count()
          assert(dirty === 1, `${testCase.name}: expected dirty worktree state`, report.failures)
        }
      }

      if (testCase.route === '/reviews') {
        const rows = await page.locator('.fd-reviews-row').count()
        caseReport.reviewRows = rows
        assert(rows >= 2, `${testCase.name}: expected review rows`, report.failures)
      }

      if (testCase.route === '/ci') {
        const commits = await page.locator('.fd-ci-commit').count()
        const checks = await page.locator('.fd-ci-check-row').count()
        caseReport.ciCommits = commits
        caseReport.ciChecks = checks
        assert(commits >= 3 && checks >= 3, `${testCase.name}: expected CI activity`, report.failures)
      }

      if (testCase.route === '/automations') {
        const rows = await page.locator('.fd-auto-row').count()
        caseReport.automationRows = rows
        assert(rows >= 2, `${testCase.name}: expected automation rules`, report.failures)
      }

      if (testCase.route === '/insights') {
        const cards = await page.locator('.fd-insight-card').count()
        const bars = await page.locator('.fd-insight-bar-row').count()
        caseReport.insightCards = cards
        caseReport.insightBars = bars
        assert(cards === 5 && bars >= 4, `${testCase.name}: expected insights`, report.failures)
      }

      if (testCase.route === '/inbox') {
        const rows = await page.locator('.fd-inbox-row').count()
        const unread = await page.locator('.fd-inbox-dot--on').count()
        caseReport.inboxRows = rows
        caseReport.inboxUnread = unread
        assert(rows >= 3 && unread >= 2, `${testCase.name}: expected inbox events`, report.failures)
      }

      if (testCase.route === '/integrations') {
        const cards = await page.locator('.fd-provider-card').count()
        caseReport.providerCards = cards
        assert(cards >= 4, `${testCase.name}: expected provider cards`, report.failures)
      }

      if (testCase.openPullRequest) {
        const title = await page.locator('.fd-pr-detail-header strong').innerText()
        caseReport.pullDetail = title
        if (
          testCase.openPullRow !== 1 &&
          !testCase.openPullFiles &&
          !testCase.openMergeDialog
        ) {
          const linked = await page.locator('.fd-pr-facts').innerText()
          assert(linked.includes('FD-142'), `${testCase.name}: linked issue missing`, report.failures)
        }
        const expectedTitle = testCase.openPullRow === 1 ? 'API Monitor' : 'repository discovery'
        assert(title.includes(expectedTitle), `${testCase.name}: PR detail missing`, report.failures)
      }

      if (testCase.openMergeDialog) {
        caseReport.mergeDialog = await page.locator('.fd-merge').isVisible()
        assert(caseReport.mergeDialog, `${testCase.name}: merge confirmation missing`, report.failures)
      }

      if (testCase.openPullFiles) {
        const files = await page.locator('.fd-pr-file').count()
        caseReport.pullFiles = files
        assert(files >= 2, `${testCase.name}: expected PR files`, report.failures)
        const warning = await page.locator('.fd-pr-warning').innerText()
        caseReport.prFilesWarning = warning
        if (testCase.openPullRow !== 1) {
          const comments = await page.locator('.fd-pr-comment').count()
          caseReport.prReviewComments = comments
          assert(comments >= 1, `${testCase.name}: expected review comments`, report.failures)
        }
        if (testCase.splitDiff) {
          const splitRows = await page.locator('.fd-pr-split__row').count()
          caseReport.splitDiffRows = splitRows
          assert(splitRows >= 2, `${testCase.name}: expected split diff rows`, report.failures)
        }
        if (testCase.openPullRow === 1) {
          assert(
            warning.includes('500') && warning.includes('3000'),
            `${testCase.name}: expected API-cap files warning`,
            report.failures,
          )
        } else {
          assert(
            warning.includes('2 of 9') && !warning.includes('3000'),
            `${testCase.name}: expected non-cap incomplete warning`,
            report.failures,
          )
        }
      }

      if (testCase.openGithubRepos) {
        const repos = await page.locator('.fd-github-repo-row').count()
        caseReport.githubRepos = repos
        assert(repos >= 2, `${testCase.name}: expected GitHub repositories`, report.failures)
      }

      if (testCase.openGithubFlow) {
        const code = await page.locator('.fd-github-device__code-block strong').innerText()
        caseReport.githubDeviceCode = code
        assert(code === 'TEST-CODE', `${testCase.name}: device code missing`, report.failures)
      }

      if (testCase.route === '/api-monitor') {
        const budgets = await page.locator('.fd-budget-panel').count()
        const queue = await page.locator('.fd-monitor-section').first().locator('.fd-monitor-row').count()
        caseReport.monitorBudgets = budgets
        caseReport.monitorQueue = queue
        assert(budgets >= 2, `${testCase.name}: expected budget panels`, report.failures)
        assert(queue >= 2, `${testCase.name}: expected queue rows`, report.failures)
      }

      if (testCase.route === '/roadmap') {
        const bars = await page.locator('.fd-roadmap__bar').count()
        caseReport.roadmapBars = bars
        assert(bars >= 2, `${testCase.name}: expected roadmap bars`, report.failures)
      }

      if (
        testCase.route === '/my-issues' &&
        !testCase.onboarding &&
        !testCase.error &&
        !testCase.applyFilter &&
        !testCase.savedView &&
        testCase.interaction !== 'chord'
      ) {
        const rows = await page.locator('[data-testid="issue-row"]').count()
        caseReport.issueRows = rows
        const boardVisible = await page.locator('.fd-board').count()
        if (boardVisible === 0 && !testCase.openCreate) {
          assert(rows >= 12, `${testCase.name}: expected ≥12 issue rows, got ${rows}`, report.failures)
        }
      }

      if (testCase.onboarding) {
        const modalWidth = await page.evaluate(() => {
          const modal = document.querySelector('.fd-modal-panel')
          return modal
            ? Math.round(modal.getBoundingClientRect().width)
            : null
        })
        caseReport.modalWidth = modalWidth
        assert(
          modalWidth !== null && modalWidth <= 560,
          `${testCase.name}: onboarding dialog ${modalWidth}px, expected ≤ 560`,
          report.failures,
        )
      }

      if (consoleErrors.length > 0) {
        report.failures.push(
          `${testCase.name}: console errors: ${consoleErrors.join(' | ')}`,
        )
      }

      report.cases.push(caseReport)
      report.screenshots.push(caseReport.screenshot)
      await context.close()
    }
  } finally {
    await browser.close()
  }

  report.assertions = report.failures.length === 0 ? 'PASS' : 'FAIL'
  writeFileSync(
    resolve(OUT_DIR, 'measurements.json'),
    `${JSON.stringify(report, null, 2)}\n`,
  )

  if (report.failures.length > 0) {
    console.error('UI Gate 0 FAIL')
    for (const failure of report.failures) {
      console.error(`- ${failure}`)
    }
    process.exitCode = 1
    return
  }

  console.log(`UI Gate 0 PASS (${report.cases.length} captures)`)
  console.log(`Artifacts: ${OUT_DIR}`)
}

await main()

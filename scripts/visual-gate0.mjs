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

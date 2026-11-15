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
    theme: 'dark',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-light-1440-expanded',
    theme: 'light',
    sidebar: 'expanded',
    width: 1440,
    height: 900,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1280-collapsed',
    theme: 'dark',
    sidebar: 'collapsed',
    width: 1280,
    height: 800,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-light-1280-collapsed',
    theme: 'light',
    sidebar: 'collapsed',
    width: 1280,
    height: 800,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1100-collapsed',
    theme: 'dark',
    sidebar: 'expanded',
    width: 1100,
    height: 700,
    route: '/my-issues',
    measure: true,
  },
  {
    name: 'gate0-shell-dark-1440-russian',
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
    resolve(OUT_DIR, 'gate0-measurements.json'),
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

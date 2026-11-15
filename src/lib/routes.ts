const ROUTE_PREFIXES = [
  '/my-issues',
  '/inbox',
  '/activity',
  '/search',
  '/projects',
  '/initiatives',
  '/cycles',
  '/roadmap',
  '/documents',
  '/repositories',
  '/pull-requests',
  '/reviews',
  '/ci',
  '/insights',
  '/dashboards',
  '/integrations',
  '/automations',
  '/api-monitor',
  '/settings',
  '/about',
]

/** Internal-only route guard for persisted navigation state. */
export function normaliseRoute(path: string): string {
  if (!path.startsWith('/') || path.startsWith('//') || path.includes('..')) {
    return '/my-issues'
  }
  const base = path.split('?')[0] ?? path
  if (base === '/') {
    return '/my-issues'
  }
  if (base.startsWith('/settings')) {
    return base
  }
  return ROUTE_PREFIXES.includes(base) ? base : '/my-issues'
}

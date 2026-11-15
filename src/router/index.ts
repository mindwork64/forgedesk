import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

import AboutView from '@/features/about/AboutView.vue'
import MyIssuesView from '@/features/issues/MyIssuesView.vue'
import PlaceholderView from '@/features/placeholder/PlaceholderView.vue'
import SettingsView from '@/features/settings/SettingsView.vue'

declare module 'vue-router' {
  interface RouteMeta {
    titleKey: string
    descriptionKey?: string
    stage?: number
    settings?: boolean
  }
}

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/my-issues' },
  {
    path: '/my-issues',
    name: 'my-issues',
    component: MyIssuesView,
    meta: {
      titleKey: 'pages.myIssues.title',
      descriptionKey: 'pages.myIssues.description',
    },
  },
  {
    path: '/inbox',
    name: 'inbox',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.inbox.title',
      descriptionKey: 'pages.inbox.description',
      stage: 7,
    },
  },
  {
    path: '/activity',
    name: 'activity',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.activity.title',
      descriptionKey: 'pages.activity.description',
      stage: 7,
    },
  },
  {
    path: '/search',
    name: 'search',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.search.title',
      descriptionKey: 'pages.search.description',
      stage: 2,
    },
  },
  {
    path: '/projects',
    name: 'projects',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.projects.title',
      descriptionKey: 'pages.projects.description',
      stage: 3,
    },
  },
  {
    path: '/initiatives',
    name: 'initiatives',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.initiatives.title',
      descriptionKey: 'pages.initiatives.description',
      stage: 3,
    },
  },
  {
    path: '/cycles',
    name: 'cycles',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.cycles.title',
      descriptionKey: 'pages.cycles.description',
      stage: 3,
    },
  },
  {
    path: '/roadmap',
    name: 'roadmap',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.roadmap.title',
      descriptionKey: 'pages.roadmap.description',
      stage: 3,
    },
  },
  {
    path: '/documents',
    name: 'documents',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.documents.title',
      descriptionKey: 'pages.documents.description',
      stage: 3,
    },
  },
  {
    path: '/repositories',
    name: 'repositories',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.repositories.title',
      descriptionKey: 'pages.repositories.description',
      stage: 4,
    },
  },
  {
    path: '/pull-requests',
    name: 'pull-requests',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.pullRequests.title',
      descriptionKey: 'pages.pullRequests.description',
      stage: 6,
    },
  },
  {
    path: '/reviews',
    name: 'reviews',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.reviews.title',
      descriptionKey: 'pages.reviews.description',
      stage: 6,
    },
  },
  {
    path: '/ci',
    name: 'ci',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.ci.title',
      descriptionKey: 'pages.ci.description',
      stage: 6,
    },
  },
  {
    path: '/insights',
    name: 'insights',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.insights.title',
      descriptionKey: 'pages.insights.description',
      stage: 7,
    },
  },
  {
    path: '/dashboards',
    name: 'dashboards',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.dashboards.title',
      descriptionKey: 'pages.dashboards.description',
      stage: 7,
    },
  },
  {
    path: '/integrations',
    name: 'integrations',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.integrations.title',
      descriptionKey: 'pages.integrations.description',
      stage: 4,
    },
  },
  {
    path: '/automations',
    name: 'automations',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.automations.title',
      descriptionKey: 'pages.automations.description',
      stage: 7,
    },
  },
  {
    path: '/api-monitor',
    name: 'api-monitor',
    component: PlaceholderView,
    meta: {
      titleKey: 'pages.apiMonitor.title',
      descriptionKey: 'pages.apiMonitor.description',
      stage: 4,
    },
  },
  {
    path: '/settings/:section?',
    name: 'settings',
    component: SettingsView,
    meta: {
      titleKey: 'pages.settings.title',
      descriptionKey: 'pages.settings.description',
      settings: true,
    },
  },
  {
    path: '/about',
    name: 'about',
    component: AboutView,
    meta: {
      titleKey: 'pages.about.title',
      descriptionKey: 'pages.about.description',
    },
  },
  { path: '/:pathMatch(.*)*', redirect: '/my-issues' },
]

export const router = createRouter({
  // Hash history keeps deep links working from the packaged custom protocol.
  history: createWebHashHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

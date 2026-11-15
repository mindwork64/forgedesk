/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
}

interface ImportMetaEnv {
  readonly VITE_VISUAL_HARNESS?: string
  readonly VITE_AI_EXTENSION?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

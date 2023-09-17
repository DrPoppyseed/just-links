/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_PUBLIC_APP_SERVER_BASE_URL: string;
  readonly VITE_PUBLIC_USER_AGENT_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

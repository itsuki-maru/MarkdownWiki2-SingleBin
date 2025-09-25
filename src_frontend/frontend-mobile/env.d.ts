/// <reference types="vite/client" />
interface ImportMetaEnv {
    readonly VITE_APP_TITLE: string,
    // その他の環境変数...
    readonly VITE_IP_ADDRESS: string
  }
  
  interface ImportMeta {
    readonly env: ImportMetaEnv
  }
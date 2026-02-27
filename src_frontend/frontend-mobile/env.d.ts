/// <reference types="vite/client" />

declare module 'qrcodejs2-fix' {
  interface QRCodeOptions {
    text?: string;
    width?: number;
    height?: number;
    colorDark?: string;
    colorLight?: string;
    correctLevel?: number;
  }
  class QRCode {
    static CorrectLevel: { L: number; M: number; Q: number; H: number };
    constructor(element: HTMLElement | string | null, options: QRCodeOptions | string);
    clear(): void;
    makeCode(text: string): void;
  }
  export default QRCode;
}

interface ImportMetaEnv {
    // その他の環境変数...
    readonly VITE_IP_ADDRESS: string
  }
  
  interface ImportMeta {
    readonly env: ImportMetaEnv
  }
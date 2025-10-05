/// <reference types="vite/client" />

declare global {
    interface Window {
        SERVER_URL: string;
        IS_WEB: boolean;
        MSG: {
            DRAG: string;
        };
        ipc: {
            // wry已经注入代码
            postMessage: (message: string) => void;
        };
        onKey: (key: string) => void;
    }
}

export {};

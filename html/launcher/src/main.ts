import { createApp } from "vue";
import "./index.css";
import App from "./App.vue";
import { setup } from "./lib/init";

if (window.IS_WEB === undefined) {
    // 如果是web环境
    window.IS_WEB = true;
}

createApp(App).mount("#app");

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
    }
}

setup();

import { createApp } from 'vue'
import './index.css'
import App from './App.vue'

window.SERVER_URL = 'http://127.0.0.1:12321';

createApp(App).mount('#app')

declare global {
    interface Window {
        SERVER_URL: string;
    }
}
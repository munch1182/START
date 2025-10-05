use key::register_any_key;
use libcommon::prelude::*;
use std::sync::LazyLock;
use window::{TaoWindow, WindowConfig, WindowFindExt, WindowManager, WryWebView};
use wry::http::Request;

static WM: LazyLock<WindowManager> = LazyLock::new(WindowManager::default);

#[logsetup]
#[tokio::main]
async fn main() -> Result<()> {
    let with_ipc = |req: Request<String>| {
        info!("ipc: {:?}", req);
        let str = req.body().to_string();
        match str.as_str() {
            "new" => {
                let _ = WM.create_new("new");
            }
            "exit" => {
                let _ = WM.exit();
            }
            "drag-window" => {
                WM.find("main", |w: &TaoWindow| w.drag_window());
            }
            _ => {}
        }
    };
    register_any_key(|key| {
        WM.find("main", |w: &WryWebView| {
            let _ = w.evaluate_script(&format!("onKey('{key}')"));
        });
    });
    let cfg = WindowConfig::new("main")
        .with_html(html().to_string())
        .with_webview(move |wb| {
            wb.with_ipc_handler(with_ipc)
                .with_initialization_script(init())
        });
    WM.create(cfg)?.run()
}

fn init() -> &'static str {
    r#"
    document.addEventListener('DOMContentLoaded', () => {
        let dragElement = document.querySelector("[drag-region]");
        dragElement.addEventListener('mousedown', (e) => {
            if (e.target.closest('button, a, input, select, textarea, [contenteditable="true"]')) {
                return;
            }
            window.ipc.postMessage('drag-window');
        });
    });
    document.addEventListener('keydown', function(e) {
        // 禁用按键
        if (e.ctrlKey) {
            e.preventDefault();
            return false;
        }
    });
    "#
}

fn html() -> impl ToString {
    r#"
<!DOCTYPE html>
<body drag-region>
    <button onclick="send('new')">新建页面</button>
    <br>
    <button onclick="send('exit')">退出</button>
    <br>
    <span>key: <span id="onKey"></span></span>
    <script>
        function send(str) {
            window.ipc.postMessage(str);
        }
        function onKey(key) {
            document.getElementById('onKey').innerHTML = key;
        }
    </script>
</body>
    "#
}

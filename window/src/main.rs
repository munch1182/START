use libcommon::prelude::{Result, debug, info, logsetup};
use std::sync::OnceLock;
use window::{App, WindowConfig, WindowCreateExt, WindowExecuteExt};

static APP: OnceLock<App> = OnceLock::new();

#[logsetup]
#[tokio::main]
pub async fn main() -> Result<()> {
    let html = r#"
<!DOCTYPE html>
<body data-tauri-drag-region>
    <button onclick="send('new')">新建页面</button>
    <br>
    <button onclick="send('exit')">退出</button>
    <script>
        function send(str) {
            window.ipc.postMessage(str);
        }
    </script>
</body>"#;

    info!("currthread: {:?}", std::thread::current().id());

    let app = App::default();
    // app.create_url("main", &index)?.run()
    let _ = APP.set(app);
    let config = WindowConfig::new("main")
        .with_html(html.to_string())
        .with_webview(|wb| {
            wb.with_ipc_handler(|req| {
                debug!("IPC: {req:?}, thread: {:?}", std::thread::current().id());
                let body = req.body().as_str();
                match body {
                    "new" => {
                        let _ = APP.wait().create_html("new", "<h1>new</h1>");
                    }
                    "exit" => {
                        APP.wait().exit();
                    }
                    _ => {}
                }
            })
        });
    APP.wait().create(config)?.run()
}

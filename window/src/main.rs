use libcommon::prelude::{Result, debug, info, logsetup};
use std::sync::OnceLock;
use window::{App, WindowConfig, WindowCreateExt, WindowExecuteExt};

static APP: OnceLock<App> = OnceLock::new();

#[logsetup]
#[tokio::main]
pub async fn main() -> Result<()> {
    let html = r#"
<!DOCTYPE html>
<body>
    <button onclick="send()">新建页面</button>
    <script>
        function send() {
            window.ipc.postMessage("new");
        }
    </script>
</body>"#;

    info!("currthread: {:?}", std::thread::current().id());

    let app = App::default();
    // app.create_url("main", &index)?.run()
    let _ = APP.set(app);
    let config = WindowConfig::new("main")
        .with_html(html)
        .with_webview(|wb| {
            wb.with_ipc_handler(|req| {
                debug!("IPC: {req:?}, thread: {:?}", std::thread::current().id());
                let body = req.body().as_str();
                if body == "new" {
                    let _ = APP.wait().create_html("new", "<h1>new</h1>");
                    debug!("IPC: new");
                }
            })
        });
    APP.wait().create(config)?.run()
}

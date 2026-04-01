use crate::event::SysWindowEvent;

/// 前端桥接对象挂载的全局变量名（内部使用，不对外暴露）
pub const BRIDGE_INTERNAL: &str = "__bridge";

/// 前端桥接对象公开的 API 名称（供应用层使用，需保持稳定）
pub const BRIDGE_PUBLIC: &str = "bridge";

/// 后端回调前端响应处理函数的方法名（挂载在内部桥接对象上）
pub const BRIDGE_HANDLER_METHOD: &str = "_handleResponse";

pub const ERROR_PARAM_NAME: &str = "error";

/// 完整的后端调用表达式，用于 evaluate_script
/// 格式：window.__bridge._handleResponse(response)
pub fn bridge_handler_call(response_json: &str) -> String {
    format!(
        "window.{}.{}({});",
        BRIDGE_INTERNAL, BRIDGE_HANDLER_METHOD, response_json
    )
}
pub(crate) fn setup_script() -> String {
    let window_commands = window_commands_script();

    format!(
        r#"
(function() {{
  const BRIDGE = {{
    _nextId: 1,
    _callbacks: new Map(),
    {handler}: function(response) {{
      response = typeof response === 'string' ? JSON.parse(response) : response;
      const cb = this._callbacks.get(response.id);
      if (cb) {{
        this._callbacks.delete(response.id);
        if (response.payload && typeof response.payload === 'object' && response.payload.{error}) {{
          cb.reject(new Error(response.payload.{error}));
        }} else {{
          cb.resolve(response.payload);
        }}
      }}
    }},
    send: function(command, payload) {{
      const id = this._nextId++;
      return new Promise((resolve, reject) => {{
        this._callbacks.set(id, {{ resolve, reject }});
        window.ipc.postMessage(JSON.stringify({{ id, command, payload }}));
      }});
    }},
    sendRaw: function(command) {{
      window.ipc.postMessage(command);
    }}
  }};
  window.{internal} = BRIDGE;
  window.{public} = {{
    send: BRIDGE.send.bind(BRIDGE),
    sendRaw: BRIDGE.sendRaw.bind(BRIDGE)
  }};

  {cmd}
}})();
        "#,
        internal = BRIDGE_INTERNAL,
        public = BRIDGE_PUBLIC,
        handler = BRIDGE_HANDLER_METHOD,
        error = ERROR_PARAM_NAME,
        cmd = window_commands,
    )
}

/// 生成窗口事件处理相关的 JavaScript 代码（可拖动元素、拖动开始、关闭/最小化）
fn window_commands_script() -> String {
    let drag_start = SysWindowEvent::DragStart.to_string();
    let close = SysWindowEvent::Close.to_string();
    let minimize = SysWindowEvent::Minimize.to_string();
    format!(
        r#"
  // 初始化可拖动元素
  const initDraggable = () => {{
    document.querySelectorAll('[data-decoration]').forEach(el => el.draggable = true);
  }};

  // 窗口拖动
  document.addEventListener('dragstart', (e) => {{
    if (e.target.closest('[data-decoration]')) {{
      e.preventDefault();
      window.{public}.sendRaw('{}');
    }}
  }});

  // 窗口关闭/最小化
  document.addEventListener('click', (e) => {{
    const btn = e.target.closest('[data-command]');
    if (!btn) return;
    const cmd = btn.getAttribute('data-command');
    e.preventDefault();
    if (cmd === '{}' || cmd === '{}') {{
      window.{public}.sendRaw(cmd);
    }} else {{
      console.warn('未知命令:', cmd);
    }}
  }});

  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', initDraggable);
  }} else {{
    initDraggable();
  }}
        "#,
        drag_start,
        close,
        minimize,
        public = BRIDGE_PUBLIC,
    )
}

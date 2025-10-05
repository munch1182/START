pub use crate::key::on_key;
use parking_lot::RwLock;
use rdev::listen;
pub use rdev::{Event, EventType, Key};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

mod key;

type KeyFn = Box<dyn Fn() + Send + Sync>;
type KeyAnyFn = Box<dyn Fn(&str) + Send + Sync>;

// 存储注册的快捷键和回调函数
static REGISTERED_KEYS: LazyLock<RwLock<HashMap<String, KeyFn>>> = LazyLock::new(Default::default);
static IS_RUNNING: AtomicBool = AtomicBool::new(false);
static ANY_KEYS: LazyLock<RwLock<Option<KeyAnyFn>>> = LazyLock::new(Default::default);

/// 注册快捷键和对应的回调函数
///
/// # 参数
/// - `key_combination`: 快捷键组合，如 `ControlLeft+KeyA`, `KeyA`
/// - `callback`: 当快捷键触发时执行的回调函数
///
/// # 示例
/// ```no_run
/// register_key("ControlLeft+KeyA", || {
///     println!("ControlLeft+KeyA");
/// });
///
/// ```
pub fn register_key(key_combination: impl ToString, callback: impl Fn() + Send + Sync + 'static) {
    let key_str = key_combination.to_string();

    {
        REGISTERED_KEYS
            .write()
            .insert(key_str.clone(), Box::new(callback));
    }

    start_listener();
}

/// 注册按键回调函数
///
/// # 示例
/// ```no_run
/// register_any_key(|key| {
///     println!("{key}");
/// });
///
/// ```
pub fn register_any_key(callback: impl Fn(&str) + Send + Sync + 'static) {
    {
        ANY_KEYS.write().replace(Box::new(callback));
    }

    start_listener();
}

pub fn unregister_any_key() {
    {
        ANY_KEYS.write().take();
    }
}

/// 移除已注册的快捷键
pub fn unregister_key(key_combination: impl ToString) {
    let key_str = key_combination.to_string();

    {
        REGISTERED_KEYS.write().remove(&key_str);
    }
}

/// 执行快捷键对应的回调函数
pub(crate) fn execute_callback(key_combination: &str) {
    {
        if let Some(callback) = REGISTERED_KEYS.read().get(key_combination) {
            callback();
        }
    }
    {
        if let Some(callback) = ANY_KEYS.read().as_ref() {
            callback(key_combination)
        }
    }
}

fn start_listener() {
    if IS_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        println!("Start listening for keys");
        start_listener_impl();
    }
}

#[cfg(feature = "tokio")]
fn start_listener_impl() {
    tokio::spawn(async {
        if let Err(err) = listen(on_key) {
            eprintln!("Error listening for keys: {err:?}");
            IS_RUNNING.store(false, Ordering::SeqCst);
        }
    });
}

#[cfg(not(feature = "tokio"))]
fn start_listener_impl() {
    std::thread::spawn(|| {
        if let Err(err) = listen(on_key) {
            eprintln!("Error listening for keys: {err:?}");
            IS_RUNNING.store(false, Ordering::SeqCst);
        }
    });
}

mod event;
mod script;
mod window;
mod wm;

use std::pin::Pin;

pub use paste::paste;
pub use tao::window::WindowBuilder;
pub use window::*;
pub use window_macro::bridge;
pub use wm::*;
pub use wry::WebViewBuilder;

pub type RawMessage = Box<serde_json::value::RawValue>;
pub type Message = serde_json::Value;

pub(crate) type FnResult = std::result::Result<Message, Box<dyn std::error::Error>>;

pub type Handler<H> =
    fn(Option<RawMessage>, WindowState<H>) -> Pin<Box<dyn Future<Output = FnResult> + Send>>;

/**
 * 将函数生成一个Handler, 以调用[bridge]生成的函数
 */
#[macro_export]
macro_rules! generate {
    ($($fn:ident),* $(,)?) => {
        [
            $(
                 $crate::paste! {
                    (stringify!($fn).to_string(), $fn::[<_ $fn _generate>] as $crate::Handler<_>)
                },
            )*
        ]
    };
}

pub use paste::paste;

#[macro_export]
macro_rules! define_host_group {
    (
        $group:ident,
        $(
            $(#[$attr:meta])*
            ($method:ident, $arg_ty:ty, $ret_ty:ty)
        ),* $(,)?
    ) => {
        $crate::paste! {
            use plugin::{Context, PluginResult, Value, async_trait, from_value, to_value};

            // 1. 宿主侧 trait
            #[async_trait]
            pub trait $group: Send + Sync {
                $(
                    $(#[$attr])*
                    async fn $method(&self, arg: $arg_ty) -> PluginResult<$ret_ty>;
                )*
            }

            // 2. 分发函数
            pub async fn [<try_dispatch_ $group:snake>]<T: $group + ?Sized>(
                host: &T,
                cmd: &str,
                args: Value,
            ) -> Option<PluginResult<Value>> {
                // 内部辅助函数，返回 PluginResult<Value>，使用 ? 简化错误处理
                async fn __dispatch<Arg: serde::de::DeserializeOwned, Ret: serde::Serialize, Fut: std::future::Future<Output = PluginResult<Ret>>>(
                    args: Value,
                    f: impl FnOnce(Arg) -> Fut,
                ) -> PluginResult<Value> {
                    let arg: Arg = from_value(args)?;          // 反序列化错误 -> Err
                    let result = f(arg).await?;               // 业务逻辑错误 -> Err
                    Ok(to_value(&result)?)                    // 序列化错误 -> Err
                }

                match cmd {
                    $(
                        stringify!($method) => {
                            Some(__dispatch(args, |arg| host.$method(arg)).await)
                        }
                    )*
                    _ => None,
                }
            }

            // 3. 插件端扩展 trait
            $(
                $(#[$attr])*
                #[async_trait]
                pub trait [<$method:camel>] {
                    async fn $method(&self, arg: $arg_ty) -> PluginResult<$ret_ty>;
                }

                #[async_trait]
                impl<T: Context + ?Sized> [<$method:camel>] for T {
                    async fn $method(&self, arg: $arg_ty) -> PluginResult<$ret_ty> {
                        let args = to_value(&arg)?;
                        let value = self.call_host(stringify!($method), args).await?;
                        from_value(value).map_err(Into::into)
                    }
                }
            )*
        }
    };
}

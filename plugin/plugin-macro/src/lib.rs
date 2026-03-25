use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ImplItemFn, ItemImpl, Signature, parse_macro_input};

/// 所有以该字符串开头的方法被视为可被调用的插件方法
const PLUGIN_START: &str = "call_";
/// 输入 JSON 对象中表示方法名的字段名
const NAME_METHOD: &str = "method";
/// 输入 JSON 对象中表示参数列表的字段名
const NAME_PRAMAS: &str = "params";

/// 属性宏，用于修饰一个 impl 块，生成对应的 `Plugin` trait 实现
///
/// 会查找包含以 `call_` 开头的异步实例方法（且第一个参数为 `&self`）;
/// 并注册到[`::plugin::Plugin::call`]中等待分发调用和参数;
/// 参数`Value`需要带有结构参数[NAME_METHOD]和[NAME_PRAMAS]。
/// # 示例
/// ```ignore
/// #[bridge]
/// impl MyPlugin {
///     // 不得带有生命周期
///     async fn call_foo(&self, arg: String) -> Result<String, Error> { ... }
///     async fn call_bar(&self) -> i32 { ... }
/// }
/// // 展开
/// #[async_trait]
/// impl ::plugin::Plugin for MyPlugin {
///     async fn call(&self, input: ::plugin::Value) -> Result<::plugin::Value,Error> {
///         // 示例代码
///         let (name, args) = input;
///         match name {
///             "call_foo" => self.call_foo(from_value(args)?).await
///             "call_bar" => Ok(self.call_foo().await)
///         }
///     }
/// }
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let ident = &input.self_ty;
    let (generics, where_clause) = (&input.generics, &input.generics.where_clause);
    let original_impl = quote! { #input };

    let method = collect_methods(&input.items);
    if method.is_empty() {
        return original_impl.into();
    }
    let matchs = match generate_matchs(method) {
        Ok(r) => r,
        Err(e) => return e.into_compile_error().into(),
    };

    quote! {
        #original_impl

        #[::plugin::async_trait]
        impl #generics ::plugin::Plugin for #ident #where_clause {
            async fn call(&self, input: ::plugin::Value) -> Result<::plugin::Value, Box<dyn std::error::Error + Send + Sync>> {
                let err = |str: String| Box::<dyn std::error::Error + Send + Sync>::from(str);
                let (method, params) = match input {
                    ::plugin::Value::Object(mut map) => {
                        let method = map.remove(#NAME_METHOD).ok_or_else(|| err(format!("no {}", #NAME_METHOD)))?;
                        let params = map.remove(#NAME_PRAMAS).ok_or_else(|| err(format!("no {}", #NAME_PRAMAS)))?;
                        (method, params)
                    }
                    _ => return Err(err(format!("input must be object with {} and {}", #NAME_METHOD, #NAME_PRAMAS)))
                };
                match method.as_str().ok_or_else(|| err(format!("{} is not string", #NAME_METHOD)))? {
                    #(#matchs,)*
                    _ => Err(err(format!("no method {}", method)))
                }
            }
        }
    }
    .into()
}

fn generate_matchs(methods: Vec<&ImplItemFn>) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut vec = Vec::new();
    for method in methods {
        let method_name = &method.sig.ident;
        let method_params: Vec<_> = method.sig.inputs.iter().skip(1).collect(); // 跳过self
        let params_len = method_params.len();
        let (params, call) = match params_len {
            0 => (quote! {}, quote! { self.#method_name().await }),
            1 => {
                let arg_ty = param2ty(method_params[0])?;
                (
                    quote! {
                        let arg: #arg_ty = ::plugin::from_value(params)?;
                    },
                    quote! {
                       self.#method_name(arg).await
                    },
                )
            }
            _ => {
                let mut arg_tys = Vec::new();
                for param in method_params {
                    let param_ty = param2ty(param)?;
                    arg_tys.push(param_ty);
                }
                let tuple_vars = (0..params_len).map(|i| format_ident!("arg{i}"));
                let tuple_vars2 = (0..params_len).map(|i| format_ident!("arg{i}"));
                (
                    quote! {
                        let (#(#tuple_vars,)*): (#(#arg_tys),*) = ::plugin::from_value(params)?;
                    },
                    quote! {
                        self.#method_name(#(#tuple_vars2),*).await
                    },
                )
            }
        };
        let call_with_result = if is_result(&method.sig) {
            quote! { #call? }
        } else {
            quote! { #call }
        };
        vec.push(quote! {
            stringify!(#method_name) => {
                #params
                let result = #call_with_result;
                Ok(::plugin::to_value(result)?)
            }
        });
    }
    Ok(vec)
}

fn is_result(sig: &Signature) -> bool {
    if let syn::ReturnType::Type(_, ty) = &sig.output
        && let syn::Type::Path(path) = ty.as_ref()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Result"
    {
        true
    } else {
        false
    }
}

fn param2ty(arg: &FnArg) -> Result<&syn::Type, syn::Error> {
    match arg {
        FnArg::Typed(pat_type) => Ok(&pat_type.ty),
        _ => Err(syn::Error::new_spanned(arg, "expected typed parameter")),
    }
}

/// 从 impl 块的所有条目中，筛选出符合条件的方法：
/// - 是方法（`ImplItem::Fn`）
/// - 名称以 `PLUGIN_START`（即 "call_"）开头
/// - 第一个参数是 `&self`（不可变引用）
fn collect_methods(items: &[ImplItem]) -> Vec<&ImplItemFn> {
    let mut vec = Vec::new();
    for item in items {
        if let ImplItem::Fn(method) = item {
            let name = method.sig.ident.to_string();
            if name.starts_with(PLUGIN_START)
                    && let Some(FnArg::Receiver(recv)) = method.sig.inputs.first() // 方法的第一个参数
                    && recv.reference.is_some() // 方法的第一个参数是self// 但不是mut self
                    && recv.mutability.is_none()
            {
                vec.push(method);
            }
        }
    }
    vec
}

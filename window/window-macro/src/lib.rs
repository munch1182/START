use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, Pat, PatTupleStruct, PatType, ReturnType, Signature, Type, parse_macro_input,
};

const WRAPPER_SUFFIX: &str = "_generate";
const NO_INPUT_PARAM: &str = "WindowState";

fn new_name(name: &Ident) -> Ident {
    Ident::new(&format!("_{name}{WRAPPER_SUFFIX}"), name.span())
}

/// 属性宏：将函数转换为 IPC 可调用的包装函数，并生成同名模块。
///
/// 原函数保持不变，生成的包装函数位于与原函数同名的模块内，可通过 `模块名::_原函数名_generate` 调用。
///
/// 返回值可以为 `serde_json::Value`，也可以是 `std::result::Result<serde_json::Value, Box<dyn std::error::Error>>`;
/// 如果返回值不是 `Result`, 要保证当前简写的 `Result` 的最后一段路径名是 `Result`;
///
/// 支持宿主状态参数：如果原函数最后一个参数类型为 `WindowState<H>`，则将其视为宿主状态，不会出现在参数结构体中，
/// 并在生成的包装函数中通过第二个参数传入。参数模式可以是 `state: WindowState<H>` 或 `WindowState(state): WindowState<H>`。
///
/// # 用法
/// ```ignore
/// #[bridge::fun]
/// pub fn add(a: i32, b: i32, state: WindowState<MyState>) -> std::result::Result<i32> {
///     // 可以使用 state 调用宿主方法
///     Ok(a + b)
/// }
///
/// #[bridge::fun]
/// pub fn list_plugins(WindowState(state): WindowState<AppState>) -> Vec<Plugin> {
///     // 解构方式同样有效，state 是内部 Arc<AppState>
///     vec![]
/// }
/// ```
///
/// 生成的代码：
/// ```ignore
/// // 原函数
/// pub fn add(a: i32, b: i32, state: WindowState<MyState>) -> std::result::Result<i32> { ... }
///
/// // 同名模块
/// pub mod add {
///     use super::*;
///     pub fn _add_generate(
///         _arg: Option<Box<serde_json::value::RawValue>>, state: WindowState<MyState>,
///     ) -> Pin<Box<dyn Future<Output = std::result::Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
///         Box::pin(async move {
///             let raw = _arg.ok_or_else(|| Box::<dyn std::error::Error>::from("need args but got none"))?;
///             #[derive(serde::Deserialize)]
///             struct Args {
///                 a: i32,
///                 b: i32,
///             }
///             let Args { a, b } = serde_json::from_str(raw.get())?;
///             let result = super::add(a, b, state).await?;
///             Ok(serde_json::to_value(result).unwrap())
///         })
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    match expand(input) {
        Ok(ts) => ts,
        Err(e) => e.into_compile_error().into(),
    }
}

/// 扩展主逻辑，返回生成的 TokenStream 或编译错误
fn expand(input: ItemFn) -> Result<TokenStream, syn::Error> {
    let sig = &input.sig;
    let ident = &sig.ident;

    // 收集所有普通参数
    let mut params = collect_args(sig)?;
    let last = params.pop();
    let (state_ty, had_state) = if let Some((_, ty)) = last
        && is_window_state(ty)
    {
        (Some(ty), true)
    } else {
        (None, false)
    };
    if !had_state && let Some(last) = last {
        params.push(last);
    }

    // 准备代码生成所需的辅助信息
    let is_async = sig.asyncness.is_some();
    let returns_result = is_return_result(sig);
    let state_arg_ts = if had_state {
        quote! { _state }
    } else {
        quote! {}
    };

    // 生成调用原函数并包装结果的代码块
    let call_body = build_call_body(&params, is_async, returns_result, ident, &state_arg_ts);

    // 生成包装函数
    let generate_name = new_name(ident);
    let (generics, state_param) = if had_state {
        (quote! {}, quote! { _state: #state_ty })
    } else {
        (quote! { <H> }, quote! { _state: ::window::WindowState<H> })
    };

    let output = quote! {
        #input
        pub mod #ident {
            use super::*;
            pub fn #generate_name #generics (_arg: Option<Box<serde_json::value::RawValue>>, #state_param)
                -> std::pin::Pin<Box<dyn Future<Output = std::result::Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>>
            {
                Box::pin(async move {
                    #call_body
                })
            }
        }
    };

    Ok(output.into())
}

/// 从函数签名中收集普通参数（名称和类型），不支持 self 接收者
fn collect_args(sig: &Signature) -> Result<Vec<(Ident, &Type)>, syn::Error> {
    let mut params = Vec::new();
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(arg, "self not supported"));
            }
            syn::FnArg::Typed(PatType { pat, ty, .. }) => collect_pats(pat, ty, &mut params)?,
        }
    }
    Ok(params)
}

/// 递归收集模式中的标识符（支持标识符和元组结构体解构）
fn collect_pats<'a>(
    pat: &Pat,
    ty: &'a Type,
    list: &mut Vec<(Ident, &'a Type)>,
) -> Result<(), syn::Error> {
    match pat {
        Pat::Ident(pat_ident) => {
            list.push((pat_ident.ident.clone(), ty));
            Ok(())
        }
        Pat::TupleStruct(PatTupleStruct { elems, .. }) => {
            for ele in elems {
                collect_pats(ele, ty, list)?;
            }
            Ok(())
        }
        _ => Err(syn::Error::new_spanned(pat, "unsupported argument type")),
    }
}

/// 判断类型是否为 WindowState<...>
fn is_window_state(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        path.path
            .segments
            .last()
            .map(|s| s.ident == NO_INPUT_PARAM)
            .unwrap_or(false)
    } else {
        false
    }
}

/// 判断函数返回类型是否为 Result
fn is_return_result(sig: &Signature) -> bool {
    if let ReturnType::Type(_, ty) = &sig.output
        && let Type::Path(path) = ty.as_ref()
    {
        return path
            .path
            .segments
            .last()
            .map(|s| s.ident == "Result")
            .unwrap_or(false);
    }
    false
}

/// 生成调用原函数并包装结果的代码块
fn build_call_body(
    params: &[(Ident, &Type)],
    is_async: bool,
    returns_result: bool,
    fn_ident: &Ident,
    state_arg_ts: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let await_ts = if is_async {
        quote! { .await }
    } else {
        quote! {}
    };
    let try_ts = if returns_result {
        quote! { ? }
    } else {
        quote! {}
    };

    // 根据参数个数生成不同的参数解析和调用代码
    if params.is_empty() {
        quote! {
            let result = super::#fn_ident(#state_arg_ts) #await_ts #try_ts;
            Ok(serde_json::json!(result))
        }
    } else {
        // 生成 Args 结构体定义, 因为前端是统一传json的
        let field_defs = params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        });
        let field_names = params.iter().map(|(name, _)| name);
        let field_names2 = params.iter().map(|(name, _)| name);
        quote! {
            let raw = _arg.ok_or_else(|| Box::<dyn std::error::Error>::from("need args but got none"))?;
            #[derive(serde::Deserialize)]
            struct Args {
                #(#field_defs,)*
            }
            let Args { #(#field_names,)* } = serde_json::from_str(raw.get())?;
            let result = super::#fn_ident(#(#field_names2,)* #state_arg_ts) #await_ts #try_ts;
            Ok(serde_json::json!(result))
        }
    }
}

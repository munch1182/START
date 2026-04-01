use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, ImplItem, ImplItemFn, ItemImpl, Pat, PatIdent, PatType, ReturnType, Signature, Type,
    TypeParamBound, TypeReference, parse_macro_input,
};

/// 所有以该字符串开头的方法被视为可被调用的插件方法
const PLUGIN_START: &str = "call_";
/// 输入 JSON 对象中表示方法名的字段名
const NAME_METHOD: &str = "method";
/// 输入 JSON 对象中表示参数列表的字段名
const NAME_PARAMS: &str = "params";

const NO_INPUT_PARAM: &str = "Context";

/// 属性宏，用于修饰一个 impl 块，生成对应的 `Plugin` trait 实现
///
/// 会查找包含以 `call_` 开头的异步实例方法（且第一个参数为 `&self`）;
/// 并注册到[`::plugin::Plugin::call`]中等待分发调用和参数;
/// 参数`Value`需要带有结构参数[NAME_METHOD]和[NAME_PARAMS]。
/// # 示例
/// ```ignore
/// #[call]
/// impl MyPlugin {
///     // 不得带有生命周期
///     async fn call_foo(&self, arg: String) -> Result<String, Error> { ... }
///     async fn call_bar(&self) -> i32 { ... }
///     async fn call_baz(&self, a: u32, b: String) -> bool { ... }
/// }
/// // 展开后自动实现 Plugin::call，支持：
/// // - 零参数：params 可为任意值（将被忽略）
/// // - 多参数：params 应为对象，字段名与参数名一致
/// // 展开
/// #[async_trait]
/// impl ::plugin::Plugin for MyPlugin {
///     async fn call(&self, input: ::plugin::Value) -> Result<::plugin::Value,Error> {
///         // 示例代码
///         let (name, args) = input;
///         match name {
///             "call_foo" => self.call_foo(from_value(args)?).await
///             "call_bar" => Ok(self.call_bar().await)
///             "call_baz" => self.call_baz(from_value(args)?).await    
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn call(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let ident = &input.self_ty;
    let (generics, where_clause) = (&input.generics, &input.generics.where_clause);
    let original_impl = quote! { #input };

    let methods = collect_methods(&input.items);
    if methods.is_empty() {
        return original_impl.into();
    }
    let match_arms = match generate_match_arms(methods) {
        Ok(r) => r,
        Err(e) => return e.into_compile_error().into(),
    };

    quote! {
        #original_impl

        #[unsafe(no_mangle)]
        pub fn plugin() -> Box<dyn ::plugin::Plugin + Send + Sync> {
            Box::new(#ident::default())
        }

        #[::plugin::async_trait]
        impl #generics ::plugin::Plugin for #ident #where_clause {
            async fn call(&self, input: ::plugin::Value, ctx: &dyn ::plugin::Context) -> Result<::plugin::Value, Box<dyn std::error::Error + Send + Sync>> {
                let err = |msg: String| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg))
                };
                let (method, params) = match input {
                    ::plugin::Value::Object(mut map) => {
                        let method = map.remove(#NAME_METHOD).ok_or_else(|| err(format!("missing field `{}`", #NAME_METHOD)))?;
                        let params = map.remove(#NAME_PARAMS).ok_or_else(|| err(format!("missing field `{}`", #NAME_PARAMS)))?;
                        (method, params)
                    }
                    _ => return Err(err("input must be an object { method, params }".to_string())),
                };
                let method_str = method.as_str()
                    .ok_or_else(|| err(format!("`{}` must be a string", #NAME_METHOD)))?;
                match method_str {
                    #(#match_arms)*
                    _ => Err(err(format!("unknown method '{}'", method_str))),
                }
            }
        }
    }
    .into()
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
                && let Some(FnArg::Receiver(recv)) = method.sig.inputs.first()
                && recv.reference.is_some()
                && recv.mutability.is_none()
            {
                vec.push(method);
            }
        }
    }
    vec
}

fn is_context_ty(ty: &Type) -> bool {
    // 展开引用
    let inner_ty = match ty {
        Type::Reference(TypeReference { elem, .. }) => elem.as_ref(),
        _ => ty,
    };
    // 展开 dyn Trait 或普通路径
    match inner_ty {
        Type::TraitObject(obj) => {
            // 检查第一个 trait bound 是否为 Context
            obj.bounds
                .first()
                .and_then(|bound| {
                    if let TypeParamBound::Trait(trait_bound) = bound {
                        trait_bound
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == NO_INPUT_PARAM)
                    } else {
                        None
                    }
                })
                .unwrap_or(false)
        }
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|s| s.ident == NO_INPUT_PARAM)
            .unwrap_or(false),
        _ => false,
    }
}

/// 生成匹配每个方法的 match 分支代码
fn generate_match_arms(
    methods: Vec<&ImplItemFn>,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut arms = Vec::new();
    for method in methods {
        let method_name = &method.sig.ident;
        let method_name_str = method_name.to_string();

        let mut had_context = false;
        let mut params = Vec::new();
        for arg in method.sig.inputs.iter().skip(1) {
            match arg {
                FnArg::Typed(PatType { pat, ty, .. }) => {
                    if is_context_ty(ty) {
                        had_context = true;
                        continue;
                    }
                    let param_name = extract_param_name(pat)?;
                    params.push((param_name, ty.as_ref()));
                }
                _ => return Err(syn::Error::new_spanned(arg, "unsupported parameter")),
            }
        }

        // 构建调用表达式
        let call_expr = if had_context {
            if params.is_empty() {
                quote! { self.#method_name(ctx).await }
            } else {
                let args = params.iter().map(|(name, _)| name);
                quote! { self.#method_name(#(#args),*, ctx).await }
            }
        } else {
            if params.is_empty() {
                quote! { self.#method_name().await }
            } else {
                let args = params.iter().map(|(name, _)| name);
                quote! { self.#method_name(#(#args),*).await }
            }
        };

        // 处理参数反序列化（多参数时生成 Args 结构体）
        let stmts = if params.is_empty() {
            quote! {{
                let result = #call_expr;
                Ok(::plugin::to_value(result)?)
            }}
        } else {
            let field_names: Vec<_> = params.iter().map(|(name, _)| name).collect();
            let field_types: Vec<_> = params.iter().map(|(_, ty)| ty).collect();
            let call_args = field_names.clone();
            quote! {{
                #[derive(::serde::Deserialize)]
                struct Args { #( #field_names: #field_types, )* }
                let Args { #(#call_args,)* } = ::plugin::from_value(params)?;
                let result = #call_expr;
                Ok(::plugin::to_value(result)?)
            }}
        };

        // 若原方法返回 Result，则保留错误链
        let final_code = if is_result(&method.sig) {
            quote! {{
                (async { #stmts }).await?
            }}
        } else {
            stmts
        };

        arms.push(quote! { #method_name_str => #final_code });
    }
    Ok(arms)
}

/// 从模式中提取标识符（支持普通标识符和元组结构体解构，但只取第一个标识符作为参数名）
fn extract_param_name(pat: &Pat) -> Result<proc_macro2::Ident, syn::Error> {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => Ok(ident.clone()),
        Pat::TupleStruct(tuple) => {
            if let Some(elem) = tuple.elems.first() {
                extract_param_name(elem)
            } else {
                Err(syn::Error::new_spanned(
                    pat,
                    "expected at least one identifier in tuple struct pattern",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            pat,
            "unsupported parameter pattern, expected identifier like `arg` or `(arg, ..)`",
        )),
    }
}

/// 判断函数返回类型是否为 Result
fn is_result(sig: &Signature) -> bool {
    if let ReturnType::Type(_, ty) = &sig.output
        && let Type::Path(path) = ty.as_ref()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Result"
    {
        true
    } else {
        false
    }
}

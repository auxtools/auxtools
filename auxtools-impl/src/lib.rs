mod versioned_struct;

use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Lit};

/// Internal attribute macro for BYOND structs whose memory layout varies across
/// builds.
///
/// Generates a concrete struct for each build layout along with accessor
/// methods that dispatch to the running build's layout. Intended solely for
/// internal use within the `auxtools` crate, because its expansion refers to
/// `crate::init` and `crate::inventory`.
#[proc_macro_attribute]
pub fn versioned(attr: TokenStream, item: TokenStream) -> TokenStream {
	versioned_struct::versioned(attr.into(), item.into()).into()
}

fn extract_args(a: &syn::FnArg) -> &syn::PatType {
	match a {
		syn::FnArg::Typed(p) => p,
		_ => panic!("Not supported on types with `self`!")
	}
}

/// Registers a `fn() -> Result<(), String>` callback to run on BYOND's thread
/// during library initialization.
///
/// Requires either `#[init(full)]` or `#[init(partial)]` (any other argument is
/// a compile error):
///
/// - `full` runs on the first initialization after the library loads and on the
///   first initialization after `auxtools_full_shutdown`, once the proc table
///   is ready and `#[hook("...")]` hooks are installed, before `partial`
///   callbacks.
/// - `partial` runs on every initialization, once per world.
///
/// Execution order among callbacks of the same kind is unspecified. Returning
/// `Err` or panicking fails initialization and causes `auxtools_init` to return
/// `FAILED (message)` to DM.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
	let init_type = syn::parse_macro_input!(attr as syn::Ident);
	let func = syn::parse_macro_input!(item as syn::ItemFn);
	let func_name = &func.sig.ident;

	let func_type = match init_type.to_string().as_str() {
		"full" => quote! { auxtools::FullInitFunc },
		"partial" => quote! { auxtools::PartialInitFunc },
		_ => return syn::Error::new(init_type.span(), "invalid init type").to_compile_error().into()
	};

	let inventory_define = quote! {
		auxtools::inventory::submit!(
			#func_type(#func_name)
		);
	};

	let code = quote! {
		#func
		#inventory_define
	};

	code.into()
}

/// Registers a `fn(&str)` callback to receive each BYOND runtime error message
/// on BYOND's thread before BYOND handles it.
///
/// BYOND text macros are stripped from the message, and panics inside the
/// callback are caught. Runtime errors caught internally by auxtools API calls
/// (such as a failed `Value::get`) are not forwarded. Remains active from the
/// first initialization until `auxtools_full_shutdown`.
#[proc_macro_attribute]
pub fn runtime_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let func = syn::parse_macro_input!(item as syn::ItemFn);
	let func_name = &func.sig.ident;

	let inventory_define = quote! {
		auxtools::inventory::submit!(
			auxtools::RuntimeErrorHook(#func_name)
		);
	};

	let code = quote! {
		#func
		#inventory_define
	};

	code.into()
}

/// Registers a `fn()` callback to run on BYOND's thread on every world shutdown
/// (`auxtools_shutdown`, `shut_down_with_world`, and
/// `init_on_next_world_load`).
///
/// Runs while BYOND data and `byond_string!` strings are still valid, making it
/// the place to drop any `Value`s kept by the library. Panics are caught and
/// ignored.
#[proc_macro_attribute]
pub fn shutdown(_: TokenStream, item: TokenStream) -> TokenStream {
	let func = syn::parse_macro_input!(item as syn::ItemFn);
	let func_name = &func.sig.ident;

	let inventory_define = quote! {
		auxtools::inventory::submit!(
			auxtools::PartialShutdownFunc(#func_name)
		);
	};

	let code = quote! {
		#func
		#inventory_define
	};

	code.into()
}

/// Registers a `fn()` callback to run during `auxtools_full_shutdown`, after
/// world shutdown completes and detours are disabled.
///
/// Runs immediately before the library may be unloaded, making it the place to
/// stop background threads spawned by the library. Panics are caught and
/// ignored.
#[proc_macro_attribute]
pub fn full_shutdown(_: TokenStream, item: TokenStream) -> TokenStream {
	let func = syn::parse_macro_input!(item as syn::ItemFn);
	let func_name = &func.sig.ident;

	let inventory_define = quote! {
		auxtools::inventory::submit!(
			#![crate = auxtools]
			auxtools::FullShutdownFunc(#func_name)
		);
	};

	let code = quote! {
		#func
		#inventory_define
	};

	code.into()
}

/// Configures whether auxtools pins its DLL in memory on Windows.
///
/// Invoke at module scope with `true` (the default) or `false`. A pinned DLL
/// stays loaded until the process exits, keeping the DLL file locked while
/// DreamDaemon runs. When unpinned, `auxtools_full_shutdown` calls
/// `FreeLibrary` to unload the DLL, so the library must stop all spawned
/// threads before `auxtools_full_shutdown` returns to avoid crashing
/// DreamDaemon. Has no effect on Linux.
#[proc_macro]
pub fn pin_dll(attr: TokenStream) -> TokenStream {
	let flag = syn::parse_macro_input!(attr as syn::LitBool);
	let code = quote! {
		use std::sync::atomic::{AtomicBool, Ordering};

		#[auxtools::ctor::ctor]
		#[cfg(windows)]
		fn set_pin_dll() {
			auxtools::PIN_DLL.store(#flag, Ordering::Relaxed);
		}
	};

	code.into()
}

/// Defines a proc hook. With a proc path, the hook is installed at init. Each
///
/// ```ignore
/// #[hook("/mob/proc/on_honked")]
/// fn on_honked(honker: Value) {
///     honker.call("laugh", &[])?;
///     Ok(Value::NULL)
/// }
/// ```
#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::ItemFn);
	let proc = syn::parse_macro_input!(attr as Option<syn::Lit>);
	let func_name = &input.sig.ident;
	let args = &input.sig.inputs;
	let args_len = args.len();

	match &input.sig.output {
		syn::ReturnType::Default => {}

		syn::ReturnType::Type(_, ty) => {
			return syn::Error::new(ty.span(), "Do not specify the return value of hooks")
				.to_compile_error()
				.into()
		}
	}

	let cthook_prelude = match proc {
		Some(Lit::Str(p)) => {
			quote! {
				auxtools::inventory::submit!({
					auxtools::CompileTimeHook{ proc_path: #p, hook: #func_name }
				});
			}
		}
		Some(other_literal) => {
			return syn::Error::new(other_literal.span(), "Hook attributes must be a string literal")
				.to_compile_error()
				.into()
		}
		None => quote! {}
	};
	let signature = quote! {
		fn #func_name(
			src: &auxtools::Value,
			usr: &auxtools::Value,
			mut args: Vec<auxtools::Value>,
		) -> auxtools::DMResult
	};

	let body = &input.block;
	let mut arg_names: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> = syn::punctuated::Punctuated::new();
	let mut proc_arg_unpacker: syn::punctuated::Punctuated<proc_macro2::TokenStream, syn::Token![,]> = syn::punctuated::Punctuated::new();

	for arg in args.iter().map(extract_args) {
		if let syn::Pat::Ident(p) = &*arg.pat {
			arg_names.push(p.ident.clone());
			let index = arg_names.len() - 1;
			proc_arg_unpacker.push(quote! {
				&args[#index]
			});
		}
	}
	let _default_null = quote! {
		#[allow(unreachable_code)]
		auxtools::Value::NULL
	};
	let result = quote! {
		#cthook_prelude
		#signature {
			if #args_len > args.len() {
				for i in 0..#args_len - args.len() {
					args.push(auxtools::Value::NULL)
				}
			}
			let (#arg_names) = (#proc_arg_unpacker);
			#body
		}
	};
	result.into()
}

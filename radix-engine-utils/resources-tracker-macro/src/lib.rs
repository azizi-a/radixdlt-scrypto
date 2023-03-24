#![allow(unused_imports)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    parse::{Parse, Parser},
    parse_quote,
    FnArg, Pat::Lit, Pat::Ident, Type::Path, Type::Reference
};

#[cfg(target_family = "unix")]
#[cfg(feature = "resource_tracker")]
use radix_engine_utils::QEMU_PLUGIN;
#[cfg(feature = "resource_tracker")]
use radix_engine_utils::data_analyzer::*;

#[cfg(not(feature = "resource_tracker"))]
#[proc_macro_attribute]
pub fn trace_resources(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}


#[cfg(target_family = "unix")]
#[cfg(feature = "resource_tracker")]
#[proc_macro_attribute]
pub fn trace_resources(attr: TokenStream, input: TokenStream) -> TokenStream {
    use radix_engine_utils::data_analyzer::{OutputParam, OutputParamValue};

    let args_parsed = syn::punctuated::Punctuated::<syn::ExprAssign, syn::Token![,]>::parse_terminated
        .parse(attr.clone())
        .expect("Wrong arguments passed");

    let mut log_ident: Vec<syn::Ident> = Vec::new();
    let mut log_expr_quote = Vec::new();
    let mut log_ident_after: Vec<syn::Ident> = Vec::new();
    let mut log_expr_after_quote = Vec::new();

    let mut additional_items: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut additional_items_after: Vec<proc_macro2::TokenStream> = Vec::new();

    // parse attributes
    for (idx, i) in args_parsed.into_iter().enumerate() {
        if let Ok(left_ident) = syn::parse::<syn::Ident>(i.left.as_ref().to_token_stream().into()) {
            let left = left_ident.to_string().clone();
            let right_arg = i.right.as_ref().to_token_stream();
            match left.as_str() {
                "info" => if let Ok(right) = syn::parse::<syn::LitStr>(right_arg.into()) {
                        let info_value = right.value();
                        additional_items.push( quote!{ OutputParam { name: "info".into(), value: OutputParamValue::Literal(#info_value.into())} } );
                    },
                "log" => if let Ok(right) = syn::parse::<syn::Ident>(right_arg.clone().into()) {
                        // log variable
                        log_ident.push(right);
                    } else if let Ok(right) = syn::parse::<syn::ExprMethodCall>(right_arg.clone().into()) {
                        // log method call result
                        let var = syn::Ident::new(&format!("arg{}", idx), Span::call_site());
                        log_expr_quote.push( quote!{ let #var = #right; } );
                        let var_s = var.to_string();
                        additional_items.push( quote!{ OutputParam { name: #var_s.into(), value: OutputParamValue::Literal(format!("{:?}", #var).into())} } );
                    } else if let Ok(right) = syn::parse::<syn::ExprBlock>(right_arg.into()) {
                        // log block result
                        let var = syn::Ident::new(&format!("arg{}", idx), Span::call_site());
                        log_expr_quote.push( quote!{ let #var = #right; } );
                        let var_s = var.to_string();
                        additional_items.push( quote!{ OutputParam { name: #var_s.into(), value: OutputParamValue::Literal(format!("{:?}", #var).into())} } );
                    } else {
                        panic!("Wrong log value type: {:?}", i.right.as_ref())
                    }
                "log_after" => if let Ok(right) = syn::parse::<syn::Ident>(right_arg.clone().into()) {
                        // log variable
                        if right == "ret" { // todo: optimise basing on function return type
                            additional_items_after.push( quote!{ OutputParam { name: "ret".into(), value: OutputParamValue::Literal(format!("{:?}", ret).into())} } );
                        } else {
                            log_ident_after.push(right);
                        }
                    } else if let Ok(right) = syn::parse::<syn::ExprMethodCall>(right_arg.clone().into()) {
                        // lob method call result
                        let var = syn::Ident::new(&format!("arg{}", idx), Span::call_site());
                        log_expr_after_quote.push( quote!{ let #var = #right; } );
                        let var_s = var.to_string();
                        additional_items_after.push( quote!{ OutputParam { name: #var_s.into(), value: OutputParamValue::Literal(format!("{:?}", #var).into())} } );
                    } else if let Ok(right) = syn::parse::<syn::ExprBlock>(right_arg.into()) {
                        // lob block result
                        let var = syn::Ident::new(&format!("arg{}", idx), Span::call_site());
                        log_expr_after_quote.push( quote!{ let #var = #right; } );
                        let var_s = var.to_string();
                        additional_items_after.push( quote!{ OutputParam { name: #var_s.into(), value: OutputParamValue::Literal(format!("{:?}", #var).into())} } );
                    } else {
                        panic!("Wrong log_after value type: {:?}", i.right.as_ref())
                    }
                s => panic!("Wrong argument: {}", s)
            }
        }
    }

    let output = if let Ok(mut item) = syn::Item::parse.parse(input.clone()) {
        match item {
            syn::Item::Fn(ref mut item_fn) => {

                let mut arg_evaluate = quote!{};
                for i in log_expr_quote {
                    arg_evaluate = quote!{ #arg_evaluate #i };
                }
                let mut arg_evaluate_after = quote!{};
                for i in log_expr_after_quote {
                    arg_evaluate_after = quote!{ #arg_evaluate_after #i };
                }

                let args_quote_array = create_params(&log_ident, item_fn.clone(), &additional_items);
                let args_after_quote_array = create_params(&log_ident_after, item_fn.clone(), &additional_items_after);
                let original_block = &mut item_fn.block;
                let fn_signature = item_fn.sig.ident.to_string();

                item_fn.block = Box::new( parse_quote! {{ 
                    use radix_engine_utils::QEMU_PLUGIN;
                    use radix_engine_utils::data_analyzer::{OutputParam, OutputParamValue};
                    #arg_evaluate;
                    #args_quote_array;
                    QEMU_PLUGIN.with(|v| {
                        v.borrow_mut().start_counting(#fn_signature, args.as_slice());
                    });
                    let ret = #original_block;
                    #arg_evaluate_after;
                    #args_after_quote_array;
                    QEMU_PLUGIN.with(|v| {
                        v.borrow_mut().stop_counting(#fn_signature, args.as_slice());
                    });
                    ret
                }} );
                item.into_token_stream()
            }
            _ => syn::Error::new_spanned(item, "#[trace_resources] is not supported for this item")
                .to_compile_error(),
        }

    } else {
        let input2 = proc_macro2::TokenStream::from(input);
        syn::Error::new_spanned(input2, "expected `fn` item").to_compile_error()
    };

    output.into()
}




#[cfg(target_family = "unix")]
#[cfg(feature = "resource_tracker")]
fn create_params( ident: &Vec<syn::Ident>, fn_sig: syn::ItemFn, additional_items: &Vec<proc_macro2::TokenStream> ) -> proc_macro2::TokenStream {
    let mut args_quote = Vec::new();
    for arg_ident in ident {
        let mut arg_found = false;
        for fn_arg in fn_sig.sig.inputs.iter() {
            match fn_arg {
                FnArg::Typed(v) => {
                    match v.pat.as_ref() {
                        Ident(pi) => {
                            if pi.ident == *arg_ident {
                                arg_found = true;
                                let var_name = pi.ident.to_string();
                                match v.ty.as_ref() {
                                    Path(tp) => { // u8..i64, bool
                                        if let Some(p) = tp.path.segments.last() {
                                            if p.ident == "u8" || p.ident == "u16" || p.ident == "u32" || p.ident == "u64" {
                                                args_quote.push( quote!{ OutputParam { name: #var_name.into(), value: OutputParamValue::NumberU64( #pi as u64 )} } );
                                                break;
                                            } else if p.ident == "i8" || p.ident == "i16" || p.ident == "i32" || p.ident == "i64" {
                                                args_quote.push( quote!{ OutputParam { name: #var_name.into(), value: OutputParamValue::NumberI64( #pi as i64 )} } );
                                                break;
                                            } else if p.ident == "bool" {
                                                args_quote.push( quote!{ OutputParam { name: #var_name.into(), value: OutputParamValue::NumberU64( #pi as u64 )} } );
                                                break;
                                            } else {
                                                args_quote.push( quote!{ OutputParam { name: #var_name.into(), value: OutputParamValue::Literal(format!("{:?}", #pi).into())} } );
                                                break;
                                            }
                                        }
                                    }
                                    Reference(tr) => { //&str
                                        match tr.elem.as_ref() {
                                            Path(tp) => {
                                                if let Some(p) = tp.path.segments.last() {
                                                    if p.ident == "str" {
                                                        args_quote.push( quote!{ OutputParam { name: #var_name.into(), value: OutputParamValue::Literal(#pi.into())} } );
                                                        break;
                                                    } else {
                                                        panic!("Not supported arg type: {}", p.ident);
                                                    }
                                                }
                                            }
                                            _ => ()
                                        }
                                    }
                                    _ => ()
                                }
                            }
                        } 
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        if !arg_found {
            panic!("Arg: {} not found", arg_ident.to_string());
        }
    }

    let mut args_quote_len = args_quote.len();
    let mut args_quote_array = quote!{};
    if args_quote_len > 0 {
        let q0 = &args_quote[0];
        args_quote_array = quote!{ #q0 };
        if args_quote_len > 1 {
            for i in 1..args_quote_len {
                let q1 = &args_quote[i];
                args_quote_array = quote!{ #args_quote_array, #q1 };
            }
        }
        for i in additional_items {
            args_quote_len += 1;
            args_quote_array = quote!{ #args_quote_array, #i };
        }
    } else if additional_items.len() > 0 {
        let q0 = &additional_items[0];
        args_quote_array = quote!{ #q0 };
        if additional_items.len() > 1 {
            for i in 1..additional_items.len() {
                let q1 = &additional_items[i];
                args_quote_array = quote!{ #args_quote_array, #q1 };
            }
        }
        args_quote_len = additional_items.len();
    }

    if args_quote_len > 0 {
        args_quote_array = quote!{ let args: [OutputParam; #args_quote_len] = [#args_quote_array] };
    } else {
        args_quote_array = quote!{ let args: [OutputParam; 0] = []; };
    }

    args_quote_array
}

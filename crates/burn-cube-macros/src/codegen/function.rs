use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, AngleBracketedGenericArguments, Ident, PathArguments};

use crate::{analysis::CodeAnalysis, codegen::base::codegen_expr};

/// Codegen for method call
pub(crate) fn codegen_expr_method_call(call: &syn::ExprMethodCall) -> TokenStream {
    quote::quote!( #call )
}

/// Codegen for a closure
pub(crate) fn codegen_closure(
    closure: &syn::ExprClosure,
    loop_level: usize,
    variable_analyses: &mut CodeAnalysis,
) -> TokenStream {
    let mut inputs = quote::quote! {};
    for input in closure.inputs.iter() {
        let ident = match input {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => panic!("Codegen: Unsupported {:?}", input),
        };
        inputs.extend(quote::quote! {
            #ident,
        });
    }

    let body = codegen_expr(closure.body.as_ref(), loop_level, variable_analyses);

    quote::quote! {
        |context, #inputs| #body
    }
}

/// Codegen for a function call
/// Maps
/// [A[::<...>]?::]^* func[::<...>] (args)
/// to
/// [A[::<...>]?::]^* func_expand[::<...>] (context, args)
pub(crate) fn codegen_call(
    call: &syn::ExprCall,
    loop_level: usize,
    variable_analyses: &mut CodeAnalysis,
) -> TokenStream {
    // We start with parsing the function path
    let path: Vec<(&Ident, Option<&AngleBracketedGenericArguments>)> = match call.func.as_ref() {
        syn::Expr::Path(expr_path) => {
            let mut path = Vec::new();
            for segment in expr_path.path.segments.iter() {
                let generics = if let PathArguments::AngleBracketed(arguments) = &segment.arguments
                {
                    Some(arguments)
                } else {
                    None
                };
                path.push((&segment.ident, generics));
            }
            path
        }
        _ => todo!("Codegen: func call {:?} not supported", call.func),
    };

    // Path
    let mut path_tokens = TokenStream::new();
    for (i, (ident, generics)) in path.iter().enumerate() {
        if i == path.len() - 1 {
            let func_name_expand = syn::Ident::new(
                format!("{ident}_expand").as_str(),
                proc_macro2::Span::call_site(),
            );
            path_tokens.extend(quote_spanned! {func_name_expand.span() => #func_name_expand });
            if let Some(generics) = generics {
                path_tokens.extend(quote_spanned! {generics.span() => #generics });
            }
        } else if let Some(generics) = generics {
            path_tokens.extend(quote_spanned! {ident.span() => #ident });
            path_tokens.extend(quote_spanned! {generics.span() => #generics :: });
        } else {
            path_tokens.extend(quote_spanned! {ident.span() => #ident :: });
        }
    }

    // Arguments
    let mut args = quote::quote! {
        context,
    };
    for argument in call.args.iter() {
        let arg = codegen_expr(argument, loop_level, variable_analyses);
        args.extend(quote::quote! { #arg, });
    }

    // Codegen
    quote::quote! {
        #path_tokens (#args)
    }
}

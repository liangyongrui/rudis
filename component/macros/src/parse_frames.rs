use quote::quote;
use syn::{DeriveInput, Field, Type};

use crate::utils;

fn parse_simple(field_type: &Type) -> Option<proc_macro2::TokenStream> {
    // &'a [u8]
    if let syn::Type::Reference(_) = field_type {
        return Some(quote!(parse.next_bytes()));
    }

    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field_type
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
            // String
            if *ident == "String" {
                return Some(quote! { parse.next_string() });
            }
            // DataType
            if *ident == "DataType" {
                return Some(quote! { crate::frame_parse::next_data_type(parse) });
            }
            // i64 or u64 or usize
            if *ident == "i64" || *ident == "u64" || *ident == "usize" {
                return Some(quote! { parse.next_int().map(|t|t as #field_type) });
            }
            if *ident == "Key" {
                return Some(quote! { parse.next_key() });
            }
            if *ident == "Box" {
                return Some(quote! { parse.next_bulk() });
            }
        }
    }
    None
}

pub fn do_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    // eprintln!("{}{:?}", *ast.ident, if ast.generics.lt_token.is_some() {"<'a>"} else {""});
    // let lt: &str = (&ast.generics).into();
    let read_token = utils::derive_get_struct_fields(ast)
        .unwrap()
        .into_iter()
        .map(|field: &Field| {
            let field_name = field.ident.as_ref();
            // 简单类型
            if let Some(res) = parse_simple(&field.ty) {
                return quote! { let #field_name = #res?; };
            }
            // 复合类型
            if let syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) = field.ty
            {
                if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
                    // Vec
                    if *ident == "Vec" {
                        if let syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) = arguments
                        {
                            if let syn::GenericArgument::Type(ty) = args.first().unwrap() {
                                let next = parse_simple(ty).unwrap();
                                return quote! {
                                    let mut #field_name = vec![#next?];
                                    loop {
                                        match #next {
                                            Ok(e) => #field_name.push(e),
                                            Err(connection::parse::ParseError::EndOfStream) => break,
                                            Err(err) => return Err(err.into()),
                                        }
                                    }
                                };
                            }
                        }
                    }
                }
            }
            // 可选类型
            todo!()
        })
        .collect::<Vec<_>>();

    let self_token = utils::derive_get_struct_fields(ast)
        .unwrap()
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref();
            quote!(#field_name,)
        })
        .collect::<proc_macro2::TokenStream>();

    let ident = &ast.ident;
    let generics = &ast.generics;
    let res = if generics.lt_token.is_some() {
        quote! {
            impl <'a> #ident <'a> {
                pub fn parse_frames(parse: &'a connection::parse::Parse<'a>) -> common::Result<Self> {
                    #(#read_token)*
                    Ok(Self {
                        #self_token
                    })
                }
            }
        }
    } else {
        quote! {
            impl #ident {
                pub fn parse_frames(parse: &connection::parse::Parse<'_>) -> common::Result<Self> {
                    #(#read_token)*
                    Ok(Self {
                        #self_token
                    })
                }
            }
        }
    };
    // eprintln!("{}", res);
    res
}

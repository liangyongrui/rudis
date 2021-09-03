#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::let_underscore_drop)]
#![allow(clippy::too_many_lines)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod utils;

#[proc_macro_derive(ParseFrames)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    do_derive(&ast).into()
}
fn do_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &ast.ident;
    let read_token = utils::derive_get_struct_fields(ast)
        .unwrap()
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref();
            let field_type = &field.ty;
            if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = field.ty {
                let mut tuple = vec![];
                for e in elems{
                    if let syn::Type::Path(syn::TypePath {
                        path: syn::Path { ref segments, .. },
                        ..
                    }) = e {
                        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                            if *ident == "i64"  {
                                tuple.push( quote! { parse.next_int()?, });
                                continue;
                            }
                        }
                    }
                    panic!("{:?} type not support", field_name.unwrap().to_string());
                }
                return quote!(let #field_name = (#(#tuple)*););
            }
            if let syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) = field.ty
            {

                if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
                    if *ident == "String" {
                        return quote! {
                            let #field_name = parse.next_string()?;
                        };
                    }
                    if *ident == "DataType" {
                        return quote! {
                            let #field_name = crate::frame_parse::next_data_type(parse)?;
                        };
                    }
                    if *ident == "i64" || *ident == "u64" {
                        return quote! {
                            let #field_name = parse.next_int()? as #field_type;
                        };
                    }
                    if *ident == "Arc" {
                        return quote! {
                            let #field_name = parse.next_key()?;
                        };
                    }
                    if *ident == "Vec" {
                        if let syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) = arguments
                        {
                            if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                path: syn::Path { ref segments, .. },
                                ..
                            })) = args.first().unwrap()
                            {
                                if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                                    if *ident == "String" {
                                        return quote! {
                                            let mut #field_name = vec![parse.next_string()?];
                                            loop {
                                                match parse.next_string() {
                                                    Ok(key) => #field_name.push(key),
                                                    Err(connection::parse::ParseError::EndOfStream) => break,
                                                    Err(err) => return Err(err.into()),
                                                }
                                            }
                                        };
                                    }
                                    if *ident == "Arc" {
                                        return quote! {
                                            let mut #field_name = vec![parse.next_key()?];
                                            loop {
                                                match parse.next_key() {
                                                    Ok(key) => #field_name.push(key),
                                                    Err(connection::parse::ParseError::EndOfStream) => break,
                                                    Err(err) => return Err(err.into()),
                                                }
                                            }
                                        };
                                    }
                                    if *ident == "DataType" {
                                        return quote! {
                                            let mut #field_name = vec![crate::frame_parse::next_data_type(parse)?];
                                            loop {
                                                match crate::frame_parse::next_data_type(parse) {
                                                    Ok(key) => #field_name.push(key),
                                                    Err(connection::parse::ParseError::EndOfStream) => break,
                                                    Err(err) => return Err(err.into()),
                                                }
                                            }
                                        };
                                    }
                                    if *ident == "u8" {
                                        return quote! {
                                            let #field_name = parse.next_key()?;
                                        };
                                    }
                                }
                            }
                        }
                    }
                    if *ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) = arguments
                        {
                            if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                path: syn::Path { ref segments, .. },
                                ..
                            })) = args.first().unwrap()
                            {
                                if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                                    if *ident == "i64" {
                                        return quote! {
                                            let mut #field_name = match parse.next_int() {
                                                    Ok(key) => Some(key),
                                                    Err(connection::parse::ParseError::EndOfStream) => None,
                                                    Err(err) => return Err(err.into()),
                                                };
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
            panic!("{:?} type not support", field_name.unwrap().to_string());
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

    quote! {
        impl #struct_name {
            pub fn parse_frames(parse: &mut connection::parse::Parse) -> common::Result<Self> {
                #(#read_token)*
                Ok(Self {
                    #self_token
                })
            }
        }
    }
}

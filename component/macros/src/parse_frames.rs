use quote::quote;
use syn::{DeriveInput, Field};

use crate::utils;

pub fn do_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    // eprintln!("{}{:?}", *ast.ident, if ast.generics.lt_token.is_some() {"<'a>"} else {""});
    // let lt: &str = (&ast.generics).into();
    let read_token = utils::derive_get_struct_fields(ast)
        .unwrap()
        .into_iter()
        .map(|field: &Field| {
            let field_name = field.ident.as_ref();
            let field_type = &field.ty;
            // &'a [u8]
            if let syn::Type::Reference(_) = field.ty {
                return quote!(let #field_name = parse.next_bytes()?;);
            }

            if let syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) = field.ty
            {
                if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
                    // String
                    if *ident == "String" {
                        return quote! {
                            let #field_name = parse.next_string()?;
                        };
                    }
                    // DataType
                    if *ident == "DataType" {
                        return quote! {
                            let #field_name = crate::frame_parse::next_data_type(parse)?;
                        };
                    }
                    // i64 or u64 or usize
                    if *ident == "i64" || *ident == "u64" || *ident == "usize" {
                        return quote! {
                            let #field_name = parse.next_int()? as #field_type;
                        };
                    }
                    if *ident == "Key" {
                        return quote! {
                            let #field_name = parse.next_key()?;
                        };
                    }
                    if *ident == "Box" {
                        return quote! {
                            let #field_name = parse.next_bulk()?;
                        };
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

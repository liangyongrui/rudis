use quote::quote;
use syn::{DeriveInput, Field, Meta, Type};

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
    let read_token = utils::derive_get_struct_fields(ast)
        .unwrap()
        .into_iter()
        .map(|field: &Field| {
            let field_name = field.ident.as_ref();
            let field_type = &field.ty;
            // 带默认值的类型
            // #[default(123)]
            if let Some(mate) = field
                .attrs
                .iter()
                .filter_map(|t| if let Ok(Meta::List(t)) = t.parse_meta() {
                    Some(t)
                } else {
                    None
                })
                .find(|t| t.path.get_ident().filter(|x|**x == "default").is_some()) {
                    let tokens = mate.nested;
                    let next = parse_simple(field_type).unwrap();
                    return quote! {
                        let mut #field_name = #tokens;
                        match #next {
                            Ok(e) => #field_name = e,
                            Err(connection::parse::ParseError::EndOfStream) => (),
                            Err(err) => return Err(err.into()),
                        }
                    }
                }

            // 简单类型
            if let Some(res) = parse_simple(field_type) {
                return quote! { let #field_name = #res?; };
            }
            // 复合类型(Tuple)
            if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = field_type {
                let mut tuple = vec![];
                for ty in elems{
                    tuple.push(parse_simple(ty).unwrap());
                }
                return quote!(let #field_name = (#(#tuple?,)*););
            }
            // 复合类型(Vec)
            if let syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) = field_type
            {
                if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
                    if *ident == "Vec" {
                        if let syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) = arguments {
                            if let syn::GenericArgument::Type(ty) = args.first().unwrap() {
                                 // vec tuple (allow empty vec)
                                if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = ty {
                                    let mut tuple1 = vec![];
                                    for ty in elems {
                                        let e = parse_simple(ty).unwrap();
                                        tuple1.push(quote!{#e?});
                                    }

                                    let mut tuple2 = vec![];
                                    let mut iter = elems.iter();
                                    let e = parse_simple(iter.next().unwrap()).unwrap();
                                    tuple2.push(quote!{{
                                        match #e {
                                            Ok(e) => e,
                                            Err(connection::parse::ParseError::EndOfStream) => break,
                                            Err(err) => return Err(err.into()),
                                        }
                                    }});
                                    for ty in iter {
                                        let e = parse_simple(ty).unwrap();
                                        tuple2.push(quote!{#e?});
                                    }
                                    return quote!({
                                        let mut #field_name = vec![(#(#tuple1?,)*)];
                                        loop {
                                            #field_name.push((#(#tuple2?,)*)),
                                        }
                                    });
                                }
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

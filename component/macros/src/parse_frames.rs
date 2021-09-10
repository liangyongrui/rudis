use quote::quote;
use syn::{DeriveInput, Field, Meta, Type, TypeReference};

use crate::utils;

fn parse_simple(field_type: &Type) -> Option<proc_macro2::TokenStream> {
    if let syn::Type::Reference(TypeReference { elem, .. }) = field_type {
        // &'a [u8]
        if let Type::Slice(_) = **elem {
            return Some(quote!(parse.next_bytes()));
        }
        // &'a str
        if let Type::Path(_) = **elem {
            return Some(quote!(parse.next_str()));
        }
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

fn parse_required_field(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref();
    let field_type = &field.ty;
    // 带默认值的类型
    // #[default(123)]
    if let Some(mate) = field
        .attrs
        .iter()
        .filter_map(|t| {
            if let Ok(Meta::List(t)) = t.parse_meta() {
                Some(t)
            } else {
                None
            }
        })
        .find(|t| t.path.get_ident().filter(|x| **x == "default").is_some())
    {
        let tokens = mate.nested;
        let next = parse_simple(field_type).unwrap();
        return quote! {
            let mut #field_name = #tokens;
            match #next {
                Ok(e) => #field_name = e,
                Err(common::connection::parse::ParseError::EndOfStream) => (),
                Err(err) => return Err(err.into()),
            }
        };
    }

    // 简单类型
    if let Some(res) = parse_simple(field_type) {
        return quote! { let #field_name = #res?; };
    }
    // 复合类型(Tuple)
    if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = field_type {
        let mut tuple = vec![];
        for ty in elems {
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
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let syn::GenericArgument::Type(ty) = args.first().unwrap() {
                        // vec tuple (allow empty vec)
                        if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = ty {
                            let mut tuple1 = vec![];
                            for ty in elems {
                                let e = parse_simple(ty).unwrap();
                                tuple1.push(quote! {#e?});
                            }

                            let mut tuple2 = vec![];
                            let mut iter = elems.iter();
                            let e = parse_simple(iter.next().unwrap()).unwrap();
                            tuple2.push(quote! {{
                                match #e {
                                    Ok(e) => e,
                                    Err(common::connection::parse::ParseError::EndOfStream) => break,
                                    Err(err) => return Err(err.into()),
                                }
                            }});
                            for ty in iter {
                                let e = parse_simple(ty).unwrap();
                                tuple2.push(quote! {#e?});
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
                                    Err(common::connection::parse::ParseError::EndOfStream) => break,
                                    Err(err) => return Err(err.into()),
                                }
                            }
                        };
                    }
                }
            }
        }
    }
    panic!("unsupport type")
}

enum FieldType {
    /// type is `bool`
    Bool,
    /// has `optional` attr
    Optional,
    /// others
    Required,
}

fn get_field_type(field: &Field) -> FieldType {
    if field
        .attrs
        .iter()
        .any(|t| t.path.segments.first().unwrap().ident == "optional")
    {
        return FieldType::Optional;
    }
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
            if *ident == "bool" {
                return FieldType::Bool;
            }
        }
    }
    FieldType::Required
}

fn parse_optional_fields(fields: &[&Field]) -> proc_macro2::TokenStream {
    if fields.is_empty() {
        return quote! {};
    }
    let mut res1 = vec![];

    // default
    for field in fields {
        let field_name = field.ident.as_ref();
        if let Some(mate) = field
            .attrs
            .iter()
            .filter_map(|t| {
                if let Ok(Meta::List(t)) = t.parse_meta() {
                    Some(t)
                } else {
                    None
                }
            })
            .find(|t| t.path.get_ident().filter(|x| **x == "default").is_some())
        {
            let tokens = mate.nested;
            res1.push(quote! { let mut #field_name = #tokens; });
        } else {
            res1.push(quote! { let mut #field_name = Default::default(); });
        }
    }

    // match arms
    let mut res2 = vec![];
    for field in fields {
        let field_name = field.ident.as_ref();
        let field_type = &field.ty;
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = field_type
        {
            if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                if *ident == "bool" {
                    let r = '"';
                    res2.push(quote! { #r#field_name#r => #field_name = true; });
                    continue;
                }
            }
        }

        if field
            .attrs
            .iter()
            .any(|t| t.path.segments.first().unwrap().ident == "optional")
        {
            let arms = quote! {
                // #(#field_type::match_arms_token(#field_name))
            };
            eprintln!("{}", arms);
            res2.push(arms);
        }
    }
    quote! {
        #(#res1)*
        loop {
            let tag = match parse.next_str() {
                Ok(e) => e,
                Err(common::connection::parse::ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            match tag.to_lowercase() {
                #(#res2)*
                others => return Err(format!("Unknown token: {}", others).into()),
            }
        }
    }
}
pub fn do_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let mut read_token = vec![];
    let fields = utils::derive_get_struct_fields(ast).unwrap().into_iter();
    let mut optional_fields = vec![];
    for field in fields {
        if matches!(get_field_type(field), FieldType::Bool | FieldType::Optional) {
            eprintln!("xxxxxxxxxxx");
            optional_fields.push(field);
            continue;
        }
        read_token.push(parse_required_field(field));
    }
    read_token.push(parse_optional_fields(&optional_fields));
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
                pub fn parse_frames(parse: &'a common::connection::parse::Parse<'a>) -> common::Result<Self> {
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
                pub fn parse_frames(parse: &common::connection::parse::Parse<'_>) -> common::Result<Self> {
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

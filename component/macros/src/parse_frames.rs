use quote::quote;
use syn::{DeriveInput, Field, Meta, Type, TypeReference};

use crate::utils;

fn is_simple(field_type: &Type) -> bool {
    if let syn::Type::Reference(TypeReference { elem, .. }) = field_type {
        if let Type::Slice(_) | Type::Path(_) = **elem {
            return true;
        }
    }

    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field_type
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
            if matches!(
                ident.to_string().as_str(),
                "String" | "DataType" | "i64" | "u64" | "usize" | "Float" | "Key" | "Box"
            ) {
                return true;
            }
        }
    }
    false
}

/// 前一个是optional，has_next才为true
fn parse_simple(field_type: &Type, has_next: bool) -> proc_macro2::TokenStream {
    if !is_simple(field_type) {
        panic!("not a simple type: {:?}", field_type);
    } else if has_next {
        return quote! { std::convert::TryInto::try_into(next?) };
    }
    if let syn::Type::Reference(TypeReference { elem, .. }) = field_type {
        // &'a [u8]
        if let Type::Slice(_) = **elem {
            return quote!(parse.next_bytes());
        }
        // &'a str
        if let Type::Path(_) = **elem {
            return quote!(parse.next_str());
        }
    }

    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field_type
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
            if *ident == "String" {
                return quote! { parse.next_string() };
            }
            if *ident == "DataType" {
                return quote! { crate::frame_parse::next_data_type(parse) };
            }
            if *ident == "i64" || *ident == "u64" || *ident == "usize" {
                return quote! { parse.next_int().map(|t|t as #field_type) };
            }
            if *ident == "Float" {
                return quote! { parse.next_float() };
            }
            if *ident == "Key" {
                return quote! { parse.next_key() };
            }
            if *ident == "Box" {
                return quote! { parse.next_bulk() };
            }
        }
    }
    panic!("known simple type: {:?}", field_type);
}

fn parse_required_field(field: &Field, mut has_next: bool) -> proc_macro2::TokenStream {
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
        let next = parse_simple(field_type, has_next);
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
    if is_simple(field_type) {
        let res = parse_simple(field_type, has_next);
        return quote! { let #field_name = #res?; };
    }
    // 复合类型(Tuple)
    if let syn::Type::Tuple(syn::TypeTuple { ref elems, .. }) = field_type {
        let mut tuple = vec![];
        for ty in elems {
            tuple.push(parse_simple(ty, has_next));
            has_next = false;
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
                                let e = parse_simple(ty, has_next);
                                has_next = false;
                                tuple1.push(quote! {#e?});
                            }

                            let mut tuple2 = vec![];
                            let mut iter = elems.iter();
                            let e = parse_simple(iter.next().unwrap(), false);
                            tuple2.push(quote! {{
                                match #e {
                                    Ok(e) => e,
                                    Err(common::connection::parse::ParseError::EndOfStream) => break,
                                    Err(err) => return Err(err.into()),
                                }
                            }});
                            for ty in iter {
                                let e = parse_simple(ty, false);
                                tuple2.push(quote! {#e?});
                            }
                            return quote! {
                                let mut #field_name = vec![(#(#tuple1,)*)];
                                loop {
                                    #field_name.push((#(#tuple2,)*));
                                }
                            };
                        }
                        let next1 = parse_simple(ty, has_next);
                        let next2 = parse_simple(ty, false);
                        return quote! {
                            let mut #field_name = vec![#next1?];
                            loop {
                                match #next2 {
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
    panic!("unsupport type: {:?}", field_type)
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
        let field_type = &field.ty;
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
            res1.push(quote! { let mut #field_name = #field_type::default(); });
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
                    let s_field_name = field_name.unwrap().to_string();
                    res2.push(quote! {
                        if #s_field_name == tag {
                            #field_name = true;
                            continue;
                        }
                    });
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
                if let Some(m) = #field_type::parse_frames(&tag, parse)? {
                    #field_name = m;
                    continue;
                }
            };
            res2.push(arms);
        }
    }
    let res = quote! {
        #(#res1)*
        let next = loop {
            match parse.next_frame() {
                Ok(f) => {
                    if let Ok(tag) = common::connection::parse::frame::to_lowercase_str(&f) {
                        #(#res2)*
                    }
                    break Ok(f);
                }
                Err(e) => break Err(e),
            }
        };
    };
    res
}
pub fn do_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let mut read_token = vec![];
    let fields = utils::derive_get_struct_fields(ast).unwrap().into_iter();
    let mut optional_fields = vec![];
    let mut has_next = false;
    for field in fields {
        if matches!(get_field_type(field), FieldType::Bool | FieldType::Optional) {
            optional_fields.push(field);
            continue;
        } else if !optional_fields.is_empty() {
            read_token.push(parse_optional_fields(&optional_fields));
            has_next = true;
            optional_fields.clear();
        }
        read_token.push(parse_required_field(field, has_next));
        has_next = false;
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

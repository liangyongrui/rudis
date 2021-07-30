use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod utils;

#[proc_macro_derive(ParseFrames)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    do_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
fn do_derive(ast: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &ast.ident;
    let read_token = utils::derive_get_struct_fields(&ast)
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
                    if *ident == "SimpleType" {
                        return quote! {
                            let #field_name = parse.next_simple_type()?;
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
                                    if *ident == "Arc" {
                                        return quote! {
                                            let mut #field_name = vec![parse.next_key()?];
                                            loop {
                                                match parse.next_key() {
                                                    Ok(key) => #field_name.push(key),
                                                    Err(crate::parse::ParseError::EndOfStream) => break,
                                                    Err(err) => return Err(err.into()),
                                                }
                                            }
                                        };
                                    }
                                    if *ident == "SimpleType" {
                                        return quote! {
                                            let mut #field_name = vec![parse.next_simple_type()?];
                                            loop {
                                                match parse.next_simple_type() {
                                                    Ok(key) => #field_name.push(key),
                                                    Err(crate::parse::ParseError::EndOfStream) => break,
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
                                    if *ident == "SimpleTypePair" {
                                        return quote! {
                                            let p_key = parse.next_simple_type()?;
                                            let p_value = parse.next_simple_type()?;
                                            let mut #field_name = vec![SimpleTypePair {key: p_key, value: p_value}];
                                            loop {
                                                match (parse.next_simple_type(), parse.next_simple_type()) {
                                                    (Ok(key), Ok(value)) => #field_name.push(SimpleTypePair {key, value}),
                                                    (Err(crate::parse::ParseError::EndOfStream), Err(crate::parse::ParseError::EndOfStream)) => break,
                                                    (Ok(_), Err(crate::parse::ParseError::EndOfStream)) => return Err("参数格式不对".to_owned().into()),
                                                    (Ok(_), Err(err)) => return Err(err.into()),
                                                    (Err(err), _) => return Err(err.into()),
                                                }
                                            }
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
                                                    Err(crate::parse::ParseError::EndOfStream) => None,
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

    let self_token = utils::derive_get_struct_fields(&ast)
        .unwrap()
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref();
            quote!(#field_name,)
        })
        .collect::<proc_macro2::TokenStream>();

    let res = quote! {
        impl #struct_name {
            pub fn parse_frames(parse: &mut crate::parse::Parse) -> crate::Result<Self> {
                #(#read_token)*
                Ok(Self {
                    #self_token
                })
            }
        }
    };
    Ok(res)
}

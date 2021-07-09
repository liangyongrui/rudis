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
            if let syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) = field.ty
            {
                if let Some(syn::PathSegment { ident, .. }) = segments.last() {
                    if *ident == "String" {
                        return quote! {
                            let #field_name = parse.next_string()?;
                        };
                    }
                    if *ident == "i64" || *ident == "u64" {
                        return quote! {
                            let #field_name = parse.next_int()? as #field_type;
                        };
                    }
                    if *ident == "Bytes" {
                        return quote! {
                            let #field_name = parse.next_bytes()? as #field_type;
                        };
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
            pub(crate) fn parse_frames(parse: &mut crate::parse::Parse) -> crate::Result<Self> {
                #(#read_token)*
                Ok(Self {
                    #self_token
                })
            }
        }
    };
    Ok(res)
}

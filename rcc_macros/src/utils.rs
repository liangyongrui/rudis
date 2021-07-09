use syn::Result;

pub fn derive_get_struct_fields(
    ast: &syn::DeriveInput,
) -> Option<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        return Some(named);
    }
    None
}

pub fn is_field_optional(field: &syn::Field) -> bool {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.last() {
            // we need to check the lat one, so xxx::Option() can work
            if ident == "Option" {
                return true;
            }
        }
    }
    false
}

pub fn extract_inner_type(field: &syn::Field, container_ident: String) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
            // we need to check the lat one, so xxx::Optional() can work
            if *ident == container_ident {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let syn::GenericArgument::Type(ty) = args.first().unwrap() {
                        return Some(ty);
                    }
                }
            }
        }
    }
    None
}

pub fn get_each_attr_name(field: &syn::Field) -> Option<Result<String>> {
    if let Some(attr) = field.attrs.last() {
        if let Ok(ref meta) = attr.parse_meta() {
            if meta.path().is_ident("builder") {
                if let syn::Meta::List(syn::MetaList { nested, .. }) = meta {
                    if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Str(lit),
                        ..
                    }))) = nested.last()
                    {
                        if path.is_ident("each") {
                            return Some(Ok(lit.value()));
                        } else {
                            return Some(Err(syn::Error::new_spanned(
                                meta,
                                r#"expected `builder(each = "...")`"#,
                            )));
                        }
                    }
                }
            }
        }
    }
    None
}

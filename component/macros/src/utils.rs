pub const fn derive_get_struct_fields(
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

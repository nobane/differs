//! `#[derive(Fields)]` implementation.

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, Type, parse_macro_input,
};

/* ------------------------------------------------------------------------- */
/* Helpers                                                                   */
/* ------------------------------------------------------------------------- */

const PREFIX_FIELD: &str = "__prefix";

fn has_skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        (attr.path().is_ident("config") || attr.path().is_ident("differs"))
            && attr
                .parse_args::<Ident>()
                .map(|id| id == "skip")
                .unwrap_or(false)
    })
}

fn is_std_string(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(tp) if tp.qself.is_none()
            && tp.path.segments.len() == 1
            && tp.path.segments.last().unwrap().ident == "String"
    )
}

fn is_primitive(ty: &Type) -> bool {
    const PRIMS: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64", "bool", "char",
    ];
    matches!(
        ty,
        Type::Path(tp) if tp.qself.is_none()
            && tp.path.segments.len() == 1
            && PRIMS.contains(&tp.path.segments.last().unwrap().ident.to_string().as_str())
    )
}

fn is_container(ty: &Type) -> bool {
    use syn::TypePath;
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };
    let Some(seg) = path.segments.last() else {
        return false;
    };
    matches!(
        seg.ident.to_string().as_str(),
        "Vec" | "HashSet" | "HashMap" | "BTreeSet" | "BTreeMap"
    )
}

fn is_leaf(ty: &Type) -> bool {
    is_primitive(ty)
        || is_std_string(ty)
        || is_container(ty)
        || matches!(
            ty,
            Type::Reference(_) | Type::Ptr(_) | Type::Tuple(_) | Type::Array(_)
        )
}

/// Determine the `<Type>Fields` ident for a nested type.
fn nested_fields_ident(ty: &Type) -> syn::Ident {
    match ty {
        Type::Path(tp) => {
            let seg = tp.path.segments.last().unwrap();
            format_ident!("{}Fields", seg.ident)
        }
        _ => format_ident!("UnknownFields"), // should not happen – fallback
    }
}

/* ------------------------------------------------------------------------- */
/* Entry‑point                                                               */
/* ------------------------------------------------------------------------- */

pub fn derive_fields_impl(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let root_fields_ident = format_ident!("{}Fields", ident);
    let prefix_ident = format_ident!("{}", PREFIX_FIELD);

    let mut root_methods = Vec::new();
    let mut extra_items = Vec::new(); // nested builder impls & proxy structs

    match data {
        Data::Struct(ds) => handle_struct(
            &root_fields_ident,
            &prefix_ident,
            &ident,
            &ds,
            &mut root_methods,
            &mut extra_items,
        ),
        Data::Enum(de) => handle_enum(
            &ident,
            &root_fields_ident,
            &prefix_ident,
            &de,
            &mut root_methods,
            &mut extra_items,
        ),
        Data::Union(_) => {
            return syn::Error::new_spanned(ident, "`Fields` cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        pub struct #root_fields_ident {
            #prefix_ident: ::std::borrow::Cow<'static, str>,
        }

        impl #root_fields_ident {
            #[inline(always)]
            pub const fn __root() -> Self {
                Self { #prefix_ident: ::std::borrow::Cow::Borrowed("") }
            }

            #(#root_methods)*
        }

        impl ::differs::AsField for #root_fields_ident {
            fn as_field(&self) -> ::differs::FieldName {
                ::differs::FieldName::from_string(self.#prefix_ident.to_string())
            }
        }

        #(#extra_items)*

        impl ::differs::HasFields for #ident {
            type Fields = #root_fields_ident;
            fn fields() -> Self::Fields { #root_fields_ident::__root() }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------- */
/* Struct handling                                                           */
/* ------------------------------------------------------------------------- */

fn handle_struct(
    root_fields_ident: &Ident,
    prefix_ident: &Ident,
    struct_ident: &Ident,
    data_struct: &DataStruct,
    methods: &mut Vec<proc_macro2::TokenStream>,
    extras: &mut Vec<proc_macro2::TokenStream>,
) {
    match &data_struct.fields {
        Fields::Named(named) => {
            for field in &named.named {
                let f_ident = field.ident.as_ref().unwrap();
                let fname = f_ident.to_string();
                if has_skip_attr(&field.attrs) {
                    continue;
                }

                if is_leaf(&field.ty) {
                    methods.push(quote_spanned! { f_ident.span() =>
                        #[allow(non_snake_case)]
                        pub fn #f_ident(&self) -> ::differs::FieldName {
                            ::differs::FieldName::join(self.#prefix_ident.as_ref(), #fname)
                        }
                    });
                } else {
                    let nested_ident = nested_fields_ident(&field.ty);
                    extras.push(quote_spanned! { f_ident.span() =>
                        impl #root_fields_ident {
                            #[allow(non_snake_case)]
                            pub fn #f_ident(&self) -> #nested_ident {
                                let p = if self.#prefix_ident.is_empty() {
                                    ::std::borrow::Cow::Borrowed(#fname)
                                } else {
                                    ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #fname))
                                };
                                #nested_ident { #prefix_ident: p }
                            }
                        }
                    });
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            for (idx, field) in unnamed.unnamed.iter().enumerate() {
                let method_ident = format_ident!("item{}", idx);
                let key = format!("item{}", idx);
                if has_skip_attr(&field.attrs) {
                    continue;
                }

                if is_leaf(&field.ty) {
                    methods.push(
                        quote_spanned! {  field.ident.as_ref().unwrap_or(struct_ident).span() =>
                            #[allow(non_snake_case)]
                            pub fn #method_ident(&self) -> ::differs::FieldName {
                                ::differs::FieldName::join(self.#prefix_ident.as_ref(), #key)
                            }
                        },
                    );
                } else {
                    let nested_ident = nested_fields_ident(&field.ty);
                    extras.push(quote_spanned! { field.ident.as_ref().unwrap_or(struct_ident).span() =>
                        impl #root_fields_ident {
                            #[allow(non_snake_case)]
                            pub fn #method_ident(&self) -> #nested_ident {
                                let p = if self.#prefix_ident.is_empty() {
                                    ::std::borrow::Cow::Borrowed(#key)
                                } else {
                                    ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #key))
                                };
                                #nested_ident { #prefix_ident: p }
                            }
                        }
                    });
                }
            }
        }
        Fields::Unit => {
            methods.push(quote! {
                pub fn self_(&self) -> ::differs::FieldName {
                    ::differs::FieldName::join(self.#prefix_ident.as_ref(), "")
                }
            });
        }
    }
}

/* ------------------------------------------------------------------------- */
/* Enum handling                                                             */
/* ------------------------------------------------------------------------- */

fn handle_enum(
    enum_ident: &Ident,
    root_fields_ident: &Ident,
    prefix_ident: &Ident,
    data_enum: &DataEnum,
    methods: &mut Vec<proc_macro2::TokenStream>,
    extras: &mut Vec<proc_macro2::TokenStream>,
) {
    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();
        let method_ident = variant_ident.clone(); // keep original variant ident (preserves case & avoids keywords)

        if has_skip_attr(&variant.attrs) {
            continue;
        }

        match &variant.fields {
            Fields::Unit => {
                methods.push(quote_spanned! { variant_ident.span() =>
                    #[allow(non_snake_case)]
                    pub fn #method_ident(&self) -> ::differs::FieldName {
                        ::differs::FieldName::join(self.#prefix_ident.as_ref(), #variant_name)
                    }
                });
            }
            Fields::Unnamed(unnamed) => {
                let proxy_ident = format_ident!("{}_{}Fields", enum_ident, variant_ident);
                let mut item_methods = Vec::new();

                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let item_fn = format_ident!("item{}", idx);
                    let item_key = format!("item{}", idx);

                    if has_skip_attr(&field.attrs) {
                        continue;
                    }

                    if is_leaf(&field.ty) {
                        item_methods.push(quote_spanned! {  field.ident.as_ref().unwrap_or(variant_ident).span() =>
                            #[allow(non_snake_case)]
                            pub fn #item_fn(&self) -> ::differs::FieldName {
                                ::differs::FieldName::join(self.#prefix_ident.as_ref(), #item_key)
                            }
                        });
                    } else {
                        let nested_ident = nested_fields_ident(&field.ty);
                        extras.push(quote_spanned! { field.ident.as_ref().unwrap_or(variant_ident).span() =>
                            impl #proxy_ident {
                                #[allow(non_snake_case)]
                                pub fn #item_fn(&self) -> #nested_ident {
                                    let p = if self.#prefix_ident.is_empty() {
                                        ::std::borrow::Cow::Borrowed(#item_key)
                                    } else {
                                        ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #item_key))
                                    };
                                    #nested_ident { #prefix_ident: p }
                                }
                            }
                        });
                    }
                }

                extras.push(quote_spanned! { variant_ident.span() =>
                    #[allow(non_camel_case_types)]
                    pub struct #proxy_ident {
                        #prefix_ident: ::std::borrow::Cow<'static, str>,
                    }
                    impl #proxy_ident { #(#item_methods)* }
                    impl ::differs::AsField for #proxy_ident {
                        fn as_field(&self) -> ::differs::FieldName {
                            ::differs::FieldName::from_string(self.#prefix_ident.to_string())
                        }
                    }
                    impl #root_fields_ident {
                        #[allow(non_snake_case)]
                        pub fn #method_ident(&self) -> #proxy_ident {
                            let p = if self.#prefix_ident.is_empty() {
                                ::std::borrow::Cow::Borrowed(#variant_name)
                            } else {
                                ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #variant_name))
                            };
                            #proxy_ident { #prefix_ident: p }
                        }
                    }
                });
            }
            Fields::Named(named) => {
                let proxy_ident = format_ident!("{}_{}Fields", enum_ident, variant_ident);
                let mut proxy_methods = Vec::new();

                for field in &named.named {
                    let f_ident = field.ident.as_ref().unwrap();
                    let fname = f_ident.to_string();
                    if has_skip_attr(&field.attrs) {
                        continue;
                    }

                    if is_leaf(&field.ty) {
                        proxy_methods.push(quote_spanned! { f_ident.span() =>
                            #[allow(non_snake_case)]
                            pub fn #f_ident(&self) -> ::differs::FieldName {
                                ::differs::FieldName::join(self.#prefix_ident.as_ref(), #fname)
                            }
                        });
                    } else {
                        let nested_ident = nested_fields_ident(&field.ty);
                        extras.push(quote_spanned! { f_ident.span() =>
                            impl #proxy_ident {
                                #[allow(non_snake_case)]
                                pub fn #f_ident(&self) -> #nested_ident {
                                    let p = if self.#prefix_ident.is_empty() {
                                        ::std::borrow::Cow::Borrowed(#fname)
                                    } else {
                                        ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #fname))
                                    };
                                    #nested_ident { #prefix_ident: p }
                                }
                            }
                        });
                    }
                }

                extras.push(quote_spanned! { variant_ident.span() =>
                    #[allow(non_camel_case_types)]
                    pub struct #proxy_ident {
                        #prefix_ident: ::std::borrow::Cow<'static, str>,
                    }
                    impl #proxy_ident { #(#proxy_methods)* }
                    impl ::differs::AsField for #proxy_ident {
                        fn as_field(&self) -> ::differs::FieldName {
                            ::differs::FieldName::from_string(self.#prefix_ident.to_string())
                        }
                    }
                    impl #root_fields_ident {
                        #[allow(non_snake_case)]
                        pub fn #method_ident(&self) -> #proxy_ident {
                            let p = if self.#prefix_ident.is_empty() {
                                ::std::borrow::Cow::Borrowed(#variant_name)
                            } else {
                                ::std::borrow::Cow::Owned(format!("{}.{}", self.#prefix_ident, #variant_name))
                            };
                            #proxy_ident { #prefix_ident: p }
                        }
                    }
                });
            }
        }
    }
}

//! **`#[derive(Diff)]`** implementation.

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments as ABGA, Attribute, Data, DeriveInput, Fields, GenericArgument,
    Ident, PathArguments, PathSegment, Type, parse_macro_input,
};

/* ------------------------------------------------------------------------- */
/* Helper predicates                                                         */
/* ------------------------------------------------------------------------- */

fn is_std_string(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(tp)
            if tp.qself.is_none()
            && tp.path.segments.last().is_some_and(
                |seg| seg.ident == "String" && tp.path.segments.len() == 1
            )
    )
}

fn is_primitive(ty: &Type) -> bool {
    const PRIMS: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64", "bool", "char",
    ];
    matches!(
        ty,
        Type::Path(tp)
            if tp.qself.is_none()
            && tp.path.segments.len()==1
            && PRIMS.contains(&tp.path.segments.last().unwrap().ident.to_string().as_str())
    )
}

fn has_skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("differs") && a.parse_args::<Ident>().is_ok_and(|id| id == "skip")
    })
}

/* ------------------------------------------------------------------------- */
/* Container helpers                                                         */
/* ------------------------------------------------------------------------- */

enum Container<'a> {
    Vec(&'a Type),
    Set(&'a Type),
    Map(&'a Type, &'a Type),
}

fn container_kind(ty: &Type) -> Option<Container<'_>> {
    let Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;

    match seg.ident.to_string().as_str() {
        "Vec" => {
            if let PathArguments::AngleBracketed(ABGA { args, .. }) = &seg.arguments {
                if let Some(GenericArgument::Type(inner)) = args.first() {
                    return Some(Container::Vec(inner));
                }
            }
        }
        "HashSet" => {
            if let PathArguments::AngleBracketed(ABGA { args, .. }) = &seg.arguments {
                if let Some(GenericArgument::Type(inner)) = args.first() {
                    return Some(Container::Set(inner));
                }
            }
        }
        "HashMap" => {
            if let PathArguments::AngleBracketed(ABGA { args, .. }) = &seg.arguments {
                let mut it = args.iter();
                if let (Some(GenericArgument::Type(k)), Some(GenericArgument::Type(v))) =
                    (it.next(), it.next())
                {
                    return Some(Container::Map(k, v));
                }
            }
        }
        _ => {}
    }
    None
}

/* ------------------------------------------------------------------------- */
/* derive(Diff) entry-point                                                  */
/* ------------------------------------------------------------------------- */

pub fn derive_diff_impl(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let Data::Struct(ds) = data else {
        return syn::Error::new_spanned(ident, "Diff can only be derived for structs")
            .to_compile_error()
            .into();
    };
    let Fields::Named(fields) = ds.fields else {
        return syn::Error::new_spanned(ident, "Diff needs named fields")
            .to_compile_error()
            .into();
    };

    /* common idents */
    let enum_ident = format_ident!("{ident}Change");
    let snapshot_ident = format_ident!("{ident}Snapshot");
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let lt = quote!('a);

    /* ------------------------------------------------------------------ */
    /* Borrow-snapshot struct                                             */
    /* ------------------------------------------------------------------ */

    let (snap_fields, snap_init): (Vec<_>, Vec<_>) = fields
        .named
        .iter()
        .map(|f| {
            let fid = f.ident.as_ref().unwrap();
            let ty = &f.ty;

            let ref_ty = if is_std_string(ty) {
                quote_spanned!(fid.span()=> ::std::borrow::Cow<'a, str>)
            } else {
                quote_spanned!(fid.span()=> &'a #ty)
            };

            let init = if is_std_string(ty) {
                quote_spanned!(fid.span()=> ::std::borrow::Cow::Borrowed(src.#fid.as_str()))
            } else {
                quote_spanned!(fid.span()=> &src.#fid)
            };

            (
                quote_spanned!(fid.span()=> #fid : #ref_ty),
                quote_spanned!(fid.span()=> #fid : #init),
            )
        })
        .unzip();

    let snapshot_def = quote! {
        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types, dead_code)]
        pub struct #snapshot_ident<'a>{ #(#snap_fields,)* }

        impl<'a> From<&'a #ident #ty_generics> for #snapshot_ident<'a>{
            fn from(src:&'a #ident #ty_generics)->Self{
                Self{ #(#snap_init,)* }
            }
        }
    };

    /* ------------------------------------------------------------------ */
    /* Build `Change` enum + diff logic                                   */
    /* ------------------------------------------------------------------ */

    let mut enum_variants = Vec::new();
    let mut diff_arms = Vec::new();

    /* whole-object snapshot */
    enum_variants.push(quote! { self_(#snapshot_ident<#lt>) });
    diff_arms.push(quote! {
        if old!=new { out.push(#enum_ident::self_(#snapshot_ident::from(new))); }
    });

    /* per-field */
    for f in &fields.named {
        let fid = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let span = fid.span();

        if has_skip_attr(&f.attrs) {
            continue;
        }

        /* container fields */
        if let Some(kind) = container_kind(ty) {
            match kind {
                /* Vec<T> */
                Container::Vec(elem_ty) => {
                    let ch_ty = quote_spanned!(span=> ::differs::Changed<#lt,#elem_ty>);
                    enum_variants.push(quote_spanned!(span=> #fid(#ch_ty)));

                    diff_arms.push(quote_spanned!(span=>{
                        use std::collections::{HashMap, HashSet};

                        let old_v = &old.#fid;
                        let new_v = &new.#fid;

                        /* map value -> queue of old indices  */
                        let mut idx_map: HashMap<&#elem_ty, Vec<usize>> = HashMap::new();
                        for (i,v) in old_v.iter().enumerate() {
                            idx_map.entry(v).or_default().push(i);
                        }

                        /* which old indices were re-used (= kept/moved) */
                        let mut reused_old : HashSet<usize> = HashSet::new();

                        /* pass 1 – walk the NEW vector and classify */
                        for (new_idx, val) in new_v.iter().enumerate() {
                            let q = idx_map.get_mut(val);

                            match q.and_then(|vec| vec.pop()) {
                                /* identical element existed before */
                                Some(old_idx) => {
                                    reused_old.insert(old_idx);

                                    if old_idx != new_idx {
                                        out.push(#enum_ident::#fid(
                                            ::differs::Changed::Moved(val, old_idx, new_idx)
                                        ));
                                    }
                                    /* same index -> no change */
                                }
                                /* entirely new value */
                                None => {
                                    out.push(#enum_ident::#fid(
                                        ::differs::Changed::AddedAt(new_idx, val, 0)
                                    ));
                                }
                            }
                        }

                        /* pass 2 – any old indices NOT re-used are removals */
                        for (old_idx, val) in old_v.iter().enumerate() {
                            if !reused_old.contains(&old_idx) {
                                out.push(#enum_ident::#fid(
                                    ::differs::Changed::RemovedAt(old_idx, val, 0)
                                ));
                            }
                        }
                    }));
                }

                /* HashSet<T> */
                Container::Set(elem_ty) => {
                    let ch_ty = quote_spanned!(span=> ::differs::Changed<#lt,#elem_ty>);
                    enum_variants.push(quote_spanned!(span=> #fid(#ch_ty)));

                    diff_arms.push(quote_spanned!(span=>{
                        for v in old.#fid.difference(&new.#fid) {
                            out.push(#enum_ident::#fid(::differs::Changed::Removed(v)));
                        }
                        for v in new.#fid.difference(&old.#fid) {
                            out.push(#enum_ident::#fid(::differs::Changed::Added(v)));
                        }
                    }));
                }

                /* HashMap<K,V> */
                Container::Map(k, v) => {
                    let ch_ty = quote_spanned!(span=> ::differs::MapChanged<#lt,#k,#v>);
                    enum_variants.push(quote_spanned!(span=> #fid(#ch_ty)));

                    diff_arms.push(quote_spanned!(span=>{
                        /* removals + modifications */
                        for (k,ov) in &old.#fid {
                            match new.#fid.get(k) {
                                None => out.push(#enum_ident::#fid(::differs::MapChanged::RemovedEntry(k,ov))),
                                Some(nv) if nv!=ov =>
                                    out.push(#enum_ident::#fid(::differs::MapChanged::ChangedEntry(k))),
                                _ => {}
                            }
                        }
                        /* pure additions */
                        for (k,nv) in &new.#fid {
                            if !old.#fid.contains_key(k) {
                                out.push(#enum_ident::#fid(::differs::MapChanged::AddedEntry(k,nv)));
                            }
                        }
                    }));
                }
            }
            continue;
        }

        /* nested struct / enum */
        let treat_as_scalar = is_std_string(ty) || is_primitive(ty);
        if !treat_as_scalar {
            if let Type::Path(tp) = ty {
                let nested_enum = tp
                    .path
                    .segments
                    .last()
                    .map(|PathSegment { ident, .. }| format_ident!("{ident}Change"))
                    .unwrap();
                enum_variants.push(quote_spanned!(span=> #fid(#nested_enum<#lt>)));
                diff_arms.push(quote_spanned!(span=>{
                    let mut _subs = Vec::new();
                    <#ty as ::differs::HasChanges>::collect_changes(&old.#fid,&new.#fid,&mut _subs);
                    out.extend(_subs.into_iter().map(#enum_ident::#fid));
                }));
                continue;
            }
        }

        /* scalar field */
        let scalar_ty = if is_std_string(ty) {
            quote_spanned!(span=> ::std::borrow::Cow<#lt,str>)
        } else {
            quote_spanned!(span=> &#lt #ty)
        };
        enum_variants.push(quote_spanned!(span=> #fid(#scalar_ty)));

        let new_val = if is_std_string(ty) {
            quote_spanned!(span=> ::std::borrow::Cow::Borrowed(new.#fid.as_str()))
        } else {
            quote_spanned!(span=> &new.#fid)
        };
        diff_arms.push(quote_spanned!(span=>{
            if old.#fid != new.#fid {
                out.push(#enum_ident::#fid(#new_val));
            }
        }));
    }

    /* ------------------------------------------------------------------ */
    /* Emit                                                               */
    /* ------------------------------------------------------------------ */

    let expanded = quote_spanned!(ident.span()=>
        #snapshot_def

        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub enum #enum_ident<#lt>{ #( #enum_variants, )* }

        impl ::differs::HasChanges for #ident #ty_generics #where_clause {
            type Change<'a> = #enum_ident<'a> where Self:'a;
            fn collect_changes<'a>(old:&'a Self,new:&'a Self,out:&mut Vec<Self::Change<'a>>)
            where Self:'a {
                #(#diff_arms)*
            }
        }
    );

    TokenStream::from(expanded)
}

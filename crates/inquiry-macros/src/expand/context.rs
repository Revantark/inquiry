use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Path};

use crate::{
    model::{FieldInfo, ModelInfo},
    sql,
};

pub(super) struct ExpansionContext {
    pub(super) struct_name: Ident,
    pub(super) table_name: String,
    pub(super) fields: Vec<FieldInfo>,
    pub(super) query_name: Ident,
    pub(super) query_error: Ident,
    pub(super) query_condition: Ident,
    pub(super) query_group: Ident,
    pub(super) inquiry_path: Path,
    pub(super) insert_fields: String,
    pub(super) select_fields: String,
    pub(super) create_table_columns: String,
}

impl ExpansionContext {
    pub(super) fn new(model: ModelInfo) -> Self {
        let inquiry_path = inquiry_crate_path();
        let query_name = query_ident(&model.struct_name);
        let query_error = error_ident(&model.struct_name);
        let query_condition = condition_ident(&model.struct_name);
        let query_group = group_ident(&model.struct_name);
        let insert_fields = sql::insert_fields(&model.fields);
        let select_fields = sql::select_fields(&model.fields);
        let create_table_columns = sql::create_table_columns(&model.fields);

        Self {
            struct_name: model.struct_name,
            table_name: model.table_name,
            fields: model.fields,
            query_name,
            query_error,
            query_condition,
            query_group,
            inquiry_path,
            insert_fields,
            select_fields,
            create_table_columns,
        }
    }

    pub(super) fn primary_key(&self) -> Option<&FieldInfo> {
        self.fields.iter().find(|field| field.primary_key)
    }

    pub(super) fn has_non_primary_key_fields(&self) -> bool {
        self.fields.iter().any(|field| !field.primary_key)
    }
}

pub(super) fn doc_lit(text: impl AsRef<str>) -> LitStr {
    LitStr::new(text.as_ref(), proc_macro2::Span::call_site())
}

pub(super) fn field_bind_bounds(cx: &ExpansionContext) -> Vec<TokenStream> {
    cx.fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            quote! {
                for<'q> &'q #ty: ::sqlx::Encode<'q, T>,
                #ty: ::sqlx::Type<T>,
            }
        })
        .collect()
}

fn inquiry_crate_path() -> Path {
    match proc_macro_crate::crate_name("inquiry") {
        Ok(proc_macro_crate::FoundCrate::Itself) => syn::parse_quote! { ::inquiry },
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            syn::parse_quote! { ::#ident }
        }
        Err(_) => syn::parse_quote! { ::inquiry },
    }
}

fn query_ident(struct_name: &Ident) -> Ident {
    Ident::new(&format!("{}Query", struct_name), struct_name.span())
}

fn error_ident(struct_name: &Ident) -> Ident {
    Ident::new(&format!("{}QueryError", struct_name), struct_name.span())
}

fn condition_ident(struct_name: &Ident) -> Ident {
    Ident::new(
        &format!("{}QueryCondition", struct_name),
        struct_name.span(),
    )
}

fn group_ident(struct_name: &Ident) -> Ident {
    Ident::new(&format!("{}QueryGroup", struct_name), struct_name.span())
}

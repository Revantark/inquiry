use syn::{Attribute, DeriveInput, LitStr};

#[derive(Default)]
pub(crate) struct FieldAttrs {
    pub(crate) column_name: Option<String>,
    pub(crate) sql_type: Option<String>,
    pub(crate) primary_key: bool,
}

pub(crate) fn parse_table_name(input: &DeriveInput) -> syn::Result<Option<String>> {
    parse_table_name_attrs(&input.attrs)
}

fn parse_table_name_attrs(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    for attr in attrs {
        if !attr.path().is_ident("query") {
            continue;
        }

        let mut table_name = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let value: LitStr = meta.value()?.parse()?;
                table_name = Some(value.value());
                return Ok(());
            }

            Err(meta.error("unsupported query attribute; expected `table = \"...\"`"))
        })?;

        if table_name.is_some() {
            return Ok(table_name);
        }
    }

    Ok(None)
}

pub(crate) fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("query") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("column") {
                let value: LitStr = meta.value()?.parse()?;
                attrs.column_name = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("sql_type") {
                let value: LitStr = meta.value()?.parse()?;
                attrs.sql_type = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("primary_key") {
                attrs.primary_key = true;
                return Ok(());
            }

            Err(meta.error(
                "unsupported query attribute; expected `column = \"...\"`, `sql_type = \"...\"`, or `primary_key`",
            ))
        })?;
    }

    Ok(attrs)
}

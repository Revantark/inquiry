use syn::{Data, DeriveInput, Fields, Ident, Type};

use crate::{attrs, sql};

pub(crate) struct FieldInfo {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) column_name: String,
    pub(crate) sql_type: String,
    pub(crate) primary_key: bool,
}

pub(crate) struct ModelInfo {
    pub(crate) struct_name: Ident,
    pub(crate) table_name: String,
    pub(crate) fields: Vec<FieldInfo>,
}

pub(crate) fn parse(input: DeriveInput) -> syn::Result<ModelInfo> {
    let table_name =
        attrs::parse_table_name(&input.attrs)?.unwrap_or_else(|| default_table_name(&input.ident));
    validate_sql_identifier(&table_name, input.ident.span(), "table")?;
    let struct_name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => parse_fields(data.fields)?,
        _ => Vec::new(), //TODO: should handle other data type
    };

    let model = ModelInfo {
        struct_name,
        table_name,
        fields,
    };
    validate(&model)?;
    Ok(model)
}

fn parse_fields(fields: Fields) -> syn::Result<Vec<FieldInfo>> {
    let mut parsed_fields = Vec::new();

    for field in fields {
        let attrs = attrs::parse_field_attrs(&field.attrs)?;
        let sql_type = attrs
            .sql_type
            .or_else(|| sql::postgres_type_for(&field.ty))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &field.ty,
                    "unsupported field type for table creation; add #[query(sql_type = \"...\")]",
                )
            })?;

        if let Some(name) = field.ident {
            let fallback_column_name = name.to_string();
            let column_name = attrs.column_name.unwrap_or(fallback_column_name);
            validate_sql_identifier(&column_name, name.span(), "column")?;
            parsed_fields.push(FieldInfo {
                ident: name,
                ty: field.ty,
                column_name,
                sql_type,
                primary_key: attrs.primary_key,
            });
        }
    }

    Ok(parsed_fields)
}

fn default_table_name(ident: &Ident) -> String {
    ident.to_string().to_lowercase()
}

fn validate_sql_identifier(value: &str, span: proc_macro2::Span, kind: &str) -> syn::Result<()> {
    let mut chars = value.chars();
    let starts_valid = chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic());
    let rest_valid = chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric());

    if starts_valid && rest_valid {
        return Ok(());
    }

    Err(syn::Error::new(
        span,
        format!(
            "{kind} name must be a simple SQL identifier: ASCII letters, digits, and underscores, not starting with a digit"
        ),
    ))
}

fn validate(model: &ModelInfo) -> syn::Result<()> {
    let primary_key_count = model
        .fields
        .iter()
        .filter(|field| field.primary_key)
        .count();

    if primary_key_count > 1 {
        return Err(syn::Error::new(
            model.struct_name.span(),
            "only one #[query(primary_key)] field is currently supported",
        ));
    }

    Ok(())
}

use syn::Type;

use crate::model::FieldInfo;

pub(crate) fn insert_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|field| field.column_name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn select_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|field| {
            let field_name = field.ident.to_string();
            if field.column_name == field_name {
                field.column_name.clone()
            } else {
                format!("{} AS {}", field.column_name, field_name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn create_table_columns(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|field| {
            if field.primary_key {
                format!("{} {} PRIMARY KEY", field.column_name, field.sql_type)
            } else {
                format!("{} {}", field.column_name, field.sql_type)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn excluded_update_assignments(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .filter(|field| !field.primary_key)
        .map(|field| format!("{} = EXCLUDED.{}", field.column_name, field.column_name))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn postgres_type_for(ty: &Type) -> Option<String> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let ident = type_path.path.get_ident()?.to_string();
    let sql_type = match ident.as_str() {
        "String" => "TEXT",
        "i16" => "SMALLINT",
        "i32" => "INTEGER",
        "i64" => "BIGINT",
        "bool" => "BOOLEAN",
        "f32" => "REAL",
        "f64" => "DOUBLE PRECISION",
        _ => return None,
    };

    Some(sql_type.to_string())
}

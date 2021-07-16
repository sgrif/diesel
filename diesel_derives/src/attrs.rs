use proc_macro_error::{emit_warning, ResultExt};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Eq, Paren};
use syn::{parenthesized, parse, Attribute, Ident, Path, Type};

use parsers::{BelongsTo, ChangesetOptions, MysqlType, PostgresType, SqliteType};
use util::suggest_attribute;

pub enum FieldAttr {
    Embed(Ident),

    ColumnName(Ident, Ident),

    SqlType(Ident, Type),
    SerializeAs(Ident, Type),
    DeserializeAs(Ident, Type),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Eq) {
            input.parse::<Eq>()?;

            match &*name_str {
                "column_name" => Ok(FieldAttr::ColumnName(name, input.parse()?)),
                "sql_type" => Ok(FieldAttr::SqlType(name, input.parse()?)),
                "serialize_as" => Ok(FieldAttr::SerializeAs(name, input.parse()?)),
                "deserialize_as" => Ok(FieldAttr::DeserializeAs(name, input.parse()?)),
                _ => suggest_attribute(&name),
            }
        } else {
            match &*name_str {
                "embed" => Ok(FieldAttr::Embed(name)),
                _ => suggest_attribute(&name),
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum StructAttr {
    Aggregate(Ident),
    NotSized(Ident),
    ForeignDerive(Ident),

    TableName(Ident, Path),

    SqlType(Ident, Type),

    ChangesetOptions(Ident, ChangesetOptions),

    PrimaryKey(Ident, Punctuated<Ident, Comma>),

    BelongsTo(Ident, BelongsTo),

    MysqlType(Ident, MysqlType),
    SqliteType(Ident, SqliteType),
    PostgresType(Ident, PostgresType),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Eq) {
            input.parse::<Eq>()?;

            match &*name_str {
                "table_name" => Ok(StructAttr::TableName(name, input.parse()?)),
                "sql_type" => Ok(StructAttr::SqlType(name, input.parse()?)),
                _ => suggest_attribute(&name),
            }
        } else if input.peek(Paren) {
            let content;
            parenthesized!(content in input);

            match &*name_str {
                "primary_key" => Ok(StructAttr::PrimaryKey(
                    name,
                    content.parse_terminated(Ident::parse)?,
                )),
                "changeset_options" => Ok(StructAttr::ChangesetOptions(name, content.parse()?)),
                "belongs_to" => Ok(StructAttr::BelongsTo(name, content.parse()?)),
                "mysql_type" => Ok(StructAttr::MysqlType(name, content.parse()?)),
                "sqlite_type" => Ok(StructAttr::SqliteType(name, content.parse()?)),
                "postgres_type" => Ok(StructAttr::PostgresType(name, content.parse()?)),
                _ => suggest_attribute(&name),
            }
        } else {
            match &*name_str {
                "aggregate" => Ok(StructAttr::Aggregate(name)),
                "not_sized" => Ok(StructAttr::NotSized(name)),
                "foreign_derive" => Ok(StructAttr::ForeignDerive(name)),
                _ => suggest_attribute(&name),
            }
        }
    }
}

pub fn parse_attributes<T: Parse>(attrs: &[Attribute]) -> Vec<T> {
    attrs
        .iter()
        .flat_map(|attr| {
            if attr.path.is_ident("diesel") {
                attr.parse_args_with(Punctuated::<T, Comma>::parse_terminated)
                    .unwrap_or_abort()
            } else {
                parse_old_attribute(attr)
            }
        })
        .collect()
}

fn parse_old_attribute<T: Parse>(attr: &Attribute) -> Punctuated<T, Comma> {
    attr.path
        .get_ident()
        .and_then(|ident| match &*ident.to_string() {
            "table_name" | "primary_key" | "column_name" | "sql_type" | "changeset_options" => {
                emit_warning!(ident, "#[{}] attribute form is deprecated", ident);

                let Attribute { path, tokens, .. } = attr;

                Some(parse::<T>(quote! { #path #tokens }.into()).unwrap_or_abort())
            }
            _ => None,
        })
        .map_or(Punctuated::new(), |attr| {
            let mut p = Punctuated::new();
            p.push_value(attr);
            p
        })
}

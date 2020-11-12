use proc_macro_error::{abort, emit_warning};
use syn::parse::{Parse, ParseStream, Result};
use syn::token::{Comma, Eq};
use syn::{Ident, LitInt, LitStr};

use util::suggest_attribute;

pub enum PostgresType {
    Fixed(LitInt, LitInt),
    Lookup(LitStr, Option<LitStr>),
}

impl Parse for PostgresType {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut oid = None;
        let mut array_oid = None;
        let mut type_name = None;
        let mut type_schema = None;

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            input.parse::<Eq>()?;

            match &*ident.to_string() {
                "oid" => oid = Some(input.parse()?),
                "array_oid" => array_oid = Some(input.parse()?),
                "type_name" => type_name = Some(input.parse()?),
                "type_schema" => type_schema = Some(input.parse()?),
                _ => suggest_attribute(&ident),
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Comma>()?;
        }

        if let Some(type_name) = type_name {
            if oid.is_some() {
                emit_warning!(oid, "unexpected `oid` when `type_name` is present");
            }
            if array_oid.is_some() {
                emit_warning!(
                    array_oid,
                    "unexpected `array_oid` when `type_name` is present"
                );
            }

            Ok(PostgresType::Lookup(type_name, type_schema))
        } else if type_schema.is_some() {
            abort!(type_schema, "expected `type_name` to be also present");
        } else if oid.is_some() && array_oid.is_some() {
            Ok(PostgresType::Fixed(oid.unwrap(), array_oid.unwrap()))
        } else {
            abort_call_site!("expected `oid` and `array_oid` or `type_name`");
        }
    }
}

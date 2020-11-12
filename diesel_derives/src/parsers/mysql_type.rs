use syn::parse::{Parse, ParseStream, Result};
use syn::token::{Comma, Eq};
use syn::{Ident, LitStr};

use util::suggest_attribute;

pub struct MysqlType {
    pub name: LitStr,
}

impl Parse for MysqlType {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            input.parse::<Eq>()?;

            match &*ident.to_string() {
                "name" => name = Some(input.parse()?),
                _ => suggest_attribute(&ident),
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Comma>()?;
        }

        if let Some(name) = name {
            Ok(MysqlType { name })
        } else {
            abort_call_site!("expected `name`");
        }
    }
}

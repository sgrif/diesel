use syn::parse::{Parse, ParseStream, Result};
use syn::token::{Comma, Eq};
use syn::{Ident, LitBool};

use util::suggest_attribute;

pub struct ChangesetOptions {
    pub treat_none_as_null: Option<LitBool>,
}

impl Parse for ChangesetOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut treat_none_as_null = None;

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            input.parse::<Eq>()?;

            match &*ident.to_string() {
                "treat_none_as_null" => treat_none_as_null = Some(input.parse()?),
                _ => suggest_attribute(&ident),
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Comma>()?;
        }

        Ok(ChangesetOptions { treat_none_as_null })
    }
}

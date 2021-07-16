use syn::parse::{Parse, ParseStream, Result};
use syn::token::{Comma, Eq};
use syn::{Ident, TypePath};

use util::suggest_attribute;

pub struct BelongsTo {
    pub parent: TypePath,
    pub foreign_key: Option<Ident>,
}

impl Parse for BelongsTo {
    fn parse(input: ParseStream) -> Result<Self> {
        let parent = input.parse()?;
        let mut foreign_key = None;

        if input.peek(Comma) {
            input.parse::<Comma>()?;

            loop {
                if input.is_empty() {
                    break;
                }

                let ident: Ident = input.parse()?;
                input.parse::<Eq>()?;

                match &*ident.to_string() {
                    "foreign_key" => foreign_key = Some(input.parse()?),
                    _ => suggest_attribute(&ident),
                }

                if input.is_empty() {
                    break;
                }
            }
        }

        Ok(BelongsTo {
            parent,
            foreign_key,
        })
    }
}

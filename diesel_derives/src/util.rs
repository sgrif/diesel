use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, GenericArgument, Ident, Type};

use model::Model;

pub fn suggest_attribute(name: &Ident) -> ! {
    abort!(name, "`{}` is either unexpected or has bad format", name)
}

pub fn wrap_in_dummy_mod(item: TokenStream) -> TokenStream {
    quote! {
        #[allow(unused_imports)]
        const _: () = {
            // This import is not actually redundant. When using diesel_derives
            // inside of diesel, `diesel` doesn't exist as an extern crate, and
            // to work around that it contains a private
            // `mod diesel { pub use super::*; }` that this import will then
            // refer to. In all other cases, this imports refers to the extern
            // crate diesel.
            use diesel;

            #item
        };
    }
}

pub fn inner_of_option_ty(ty: &Type) -> &Type {
    option_ty_arg(ty).unwrap_or(ty)
}

pub fn is_option_ty(ty: &Type) -> bool {
    option_ty_arg(ty).is_some()
}

fn option_ty_arg(ty: &Type) -> Option<&Type> {
    use syn::PathArguments::AngleBracketed;

    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty.path.segments.iter().last().unwrap();
            match last_segment.arguments {
                AngleBracketed(ref args) if last_segment.ident == "Option" => {
                    match args.args.iter().last() {
                        Some(&GenericArgument::Type(ref ty)) => Some(ty),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn ty_for_foreign_derive(item: &DeriveInput, model: &Model) -> Type {
    if model.foreign_derive {
        match item.data {
            Data::Struct(ref body) => match body.fields.iter().next() {
                Some(field) => field.ty.clone(),
                None => abort_call_site!("foreign_derive requires at least one field"),
            },
            _ => abort_call_site!("foreign_derive can only be used with structs"),
        }
    } else {
        let ident = &item.ident;
        let (_, ty_generics, ..) = item.generics.split_for_impl();
        parse_quote!(#ident #ty_generics)
    }
}

pub fn camel_to_snake(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    result.push_str(&name[..1].to_lowercase());
    for character in name[1..].chars() {
        if character.is_uppercase() {
            result.push('_');
            for lowercase in character.to_lowercase() {
                result.push(lowercase);
            }
        } else {
            result.push(character);
        }
    }
    result
}

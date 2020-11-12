use proc_macro2::TokenStream;
use syn::DeriveInput;

use model::Model;
use util::{ty_for_foreign_derive, wrap_in_dummy_mod};

pub fn derive(mut item: DeriveInput) -> TokenStream {
    let model = Model::from_item(&item, true);
    let struct_ty = ty_for_foreign_derive(&item, &model);

    item.generics.params.push(parse_quote!(__ST));
    item.generics.params.push(parse_quote!(__DB));
    {
        let where_clause = item
            .generics
            .where_clause
            .get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(__DB: diesel::backend::Backend));
        where_clause
            .predicates
            .push(parse_quote!(__ST: diesel::sql_types::SingleValue));
        where_clause
            .predicates
            .push(parse_quote!(Self: FromSql<__ST, __DB>));
    }
    let (impl_generics, _, where_clause) = item.generics.split_for_impl();

    wrap_in_dummy_mod(quote! {
        use diesel::deserialize::{self, FromSql, Queryable};

        impl #impl_generics Queryable<__ST, __DB> for #struct_ty
        #where_clause
        {
            type Row = Self;

            fn build(row: Self::Row) -> deserialize::Result<Self> {
                Ok(row)
            }
        }
    })
}

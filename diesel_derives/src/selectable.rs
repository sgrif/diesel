use proc_macro2::TokenStream;
use syn::DeriveInput;

use field::Field;
use model::Model;
use util::wrap_in_dummy_mod;

pub fn derive(item: DeriveInput) -> TokenStream {
    let model = Model::from_item(&item, false);

    let (_, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics
        .params
        .push(parse_quote!(__DB: diesel::backend::Backend));
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let struct_name = &item.ident;

    let field_columns_ty = model.fields().iter().map(|f| field_column_ty(f, &model));
    let field_columns_inst = model.fields().iter().map(|f| field_column_inst(f, &model));

    wrap_in_dummy_mod(quote! {
        use diesel::expression::Selectable;

        impl #impl_generics Selectable<__DB>
            for #struct_name #ty_generics
        #where_clause
        {
            type SelectExpression = (#(#field_columns_ty,)*);

            fn construct_selection() -> Self::SelectExpression {
                (#(#field_columns_inst,)*)
            }
        }
    })
}

fn field_column_ty(field: &Field, model: &Model) -> TokenStream {
    if field.embed {
        let embed_ty = &field.ty;
        quote!(<#embed_ty as Selectable<__DB>>::SelectExpression)
    } else {
        let table_name = model.table_name();
        let column_name = field.column_name();
        quote!(#table_name::#column_name)
    }
}

fn field_column_inst(field: &Field, model: &Model) -> TokenStream {
    if field.embed {
        let embed_ty = &field.ty;
        quote!(<#embed_ty as Selectable<__DB>>::construct_selection())
    } else {
        let table_name = model.table_name();
        let column_name = field.column_name();
        quote!(#table_name::#column_name)
    }
}

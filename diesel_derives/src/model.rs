use proc_macro2::Span;
use syn;

use diagnostic_shim::*;
use field::*;
use meta::*;
use resolved_at_shim::*;

pub struct Model {
    pub name: syn::Ident,
    pub primary_key_names: Vec<syn::Ident>,
    table_name_from_attribute: Option<syn::Ident>,
    fields: Vec<Field>,
}

impl Model {
    pub fn from_item(item: &syn::DeriveInput) -> Result<Self, Diagnostic> {
        let table_name_from_attribute =
            MetaItem::with_name(&item.attrs, "table_name").map(|m| m.expect_ident_value());
        let primary_key_names = MetaItem::with_name(&item.attrs, "primary_key")
            .map(|m| Ok(m.nested()?.map(|m| m.expect_word()).collect()))
            .unwrap_or_else(|| Ok(vec!["id".into()]))?;
        let fields = fields_from_item_data(&item.data)?;
        Ok(Self {
            name: item.ident,
            table_name_from_attribute,
            primary_key_names,
            fields,
        })
    }

    pub fn table_name(&self) -> syn::Ident {
        self.table_name_from_attribute.unwrap_or_else(|| {
            syn::Ident::new(
                &infer_table_name(self.name.as_ref()),
                self.name.span().resolved_at(Span::call_site()),
            )
        })
    }

    pub fn dummy_mod_name(&self, trait_name: &str) -> syn::Ident {
        let name = self.name.as_ref().to_lowercase();
        format!("_impl_{}_for_{}", trait_name, name).into()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn find_column(&self, column_name: syn::Ident) -> Result<&Field, Diagnostic> {
        self.fields()
            .iter()
            .find(|f| f.column_name() == column_name)
            .ok_or_else(|| {
                column_name
                    .span()
                    .error(format!("No field with column name {}", column_name))
            })
    }

    pub fn has_table_name_attribute(&self) -> bool {
        self.table_name_from_attribute.is_some()
    }

    pub fn sql_type_of(&self, field: &Field) -> syn::Type {
        let table_name = self.table_name();
        let column_name = field.column_name();

        match field.sql_type {
            Some(ref st) => st.clone(),
            None => if self.has_table_name_attribute() {
                parse_quote!(diesel::dsl::SqlTypeOf<#table_name::#column_name>)
            } else {
                let field_name = match field.name {
                    FieldName::Named(ref x) => x.as_ref(),
                    _ => "field",
                };
                field
                    .span
                    .error(format!("Cannot determine the SQL type of {}", field_name))
                    .help(
                        "Your struct must either be annotated with `#[table_name = \"foo\"]` \
                         or have all of its fields annotated with `#[sql_type = \"Integer\"]`",
                    )
                    .emit();
                parse_quote!(())
            },
        }
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

fn infer_table_name(name: &str) -> String {
    let mut result = camel_to_snake(name);
    result.push('s');
    result
}

fn fields_from_item_data(data: &syn::Data) -> Result<Vec<Field>, Diagnostic> {
    use syn::Data::*;

    let struct_data = match *data {
        Struct(ref d) => d,
        _ => return Err(Span::call_site().error("This derive can only be used on structs")),
    };
    Ok(struct_data
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| Field::from_struct_field(f, i))
        .collect())
}

#[test]
fn infer_table_name_pluralizes_and_downcases() {
    assert_eq!("foos", &infer_table_name("Foo"));
    assert_eq!("bars", &infer_table_name("Bar"));
}

#[test]
fn infer_table_name_properly_handles_underscores() {
    assert_eq!("foo_bars", &infer_table_name("FooBar"));
    assert_eq!("foo_bar_bazs", &infer_table_name("FooBarBaz"));
}

use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, ItemStruct};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(ppproperly))]
#[darling(default)]
struct Args {
    len_for: Option<String>,
    discriminant_for: Option<String>,
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let ast: ItemStruct = parse(item).unwrap();
    let name = ast.ident;

    let serializers = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("should be a names struct");

        quote!(
            self.#field_name.serialize(w)?;
        )
    });

    quote!(
        impl Serialize for #name {
            fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<()> {
                #(#serializers) *

                Ok(())
            }
        }
    )
    .into()
}

#[proc_macro_derive(Deserialize, attributes(ppproperly))]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let ast: ItemStruct = parse(item).unwrap();
    let name = ast.ident;

    let deserializers = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("should be a names struct");

        let args = Args::from_attributes(&field.attrs).unwrap();

        quote!(
            self.#field_name.deserialize(r)?;
        )
    });

    quote!(
        impl Deserialize for #name {
            fn deserialize<R: std::io::Read>(&mut self, r: &mut R) -> Result<()> {
                #(#deserializers) *

                Ok(())
            }
        }
    )
    .into()
}

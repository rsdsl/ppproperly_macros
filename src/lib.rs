use darling::FromAttributes;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
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
        let mut out = TokenStream2::new();

        let field_name = field.ident.as_ref().expect("should be a names struct");

        let args = Args::from_attributes(&field.attrs).unwrap();

        out.extend(
            vec![quote!(
                let mut len_for = HashMap::new();
                let mut discriminant_for = HashMap::new();
            )]
            .into_iter(),
        );

        if let Some(attr) = args.len_for {
            out.extend(
                vec![quote!(
                    let mut len = 0u16;
                    len.deserialize(r)?;

                    len_for.insert(#attr, len);
                )]
                .into_iter(),
            );
        }

        if let Some(attr) = args.discriminant_for {
            out.extend(
                vec![quote!(
                    let mut discriminant = 0u8;
                    discriminant.deserialize(r)?;

                    discriminant_for.insert(#attr, discriminant);
                )]
                .into_iter(),
            );
        }

        out.extend(
            vec![quote!(
                let r = if let Some(attr) = len_for.get(field_name) {
                    r.take(attr)
                } else {
                    r
                };
            )]
            .into_iter(),
        );

        out.extend(
            vec![quote!(
                if let Some(attr) = discriminant_for.get(field_name) {
                    self.#field_name.deserialize_with_discriminant(r, attr)?;
                } else {
                    self.#field_name.deserialize(r)?;
                }
            )]
            .into_iter(),
        );

        out
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

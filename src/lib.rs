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

    let mut len_for = std::collections::HashMap::new();
    let mut discriminant_for = std::collections::HashMap::new();

    let has_len_annotations = ast.fields.iter().any(|field| {
        let args = Args::from_attributes(&field.attrs).unwrap();
        args.len_for.is_some()
    });

    let has_discriminant_annotations = ast.fields.iter().any(|field| {
        let args = Args::from_attributes(&field.attrs).unwrap();
        args.discriminant_for.is_some()
    });

    let mut map_declarations = TokenStream2::new();

    if has_len_annotations {
        map_declarations.extend(
            vec![quote!(
                let mut len_for = std::collections::HashMap::new();
            )]
            .into_iter(),
        );
    }

    if has_discriminant_annotations {
        map_declarations.extend(
            vec![quote!(
                let mut discriminant_for = std::collections::HashMap::new();
            )]
            .into_iter(),
        );
    }

    let deserializers = ast.fields.iter().map(|field| {
        let mut out = TokenStream2::new();

        let field_name = field.ident.as_ref().expect("should be a names struct");

        let args = Args::from_attributes(&field.attrs).unwrap();

        if let Some(attr) = args.len_for {
            out.extend(
                vec![quote!(
                    let mut len = 0u16;
                    len.deserialize(r)?;

                    len_for.insert(#attr, len);
                )]
                .into_iter(),
            );

            len_for.insert(attr, ());
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

            discriminant_for.insert(attr, ());
        }

        if len_for.contains_key(&field_name.to_string()) {
            out.extend(
                vec![quote!(
                    let r = r.take(len_for.get("#field_name").unwrap());
                )]
                .into_iter(),
            );
        }

        if discriminant_for.contains_key(&field_name.to_string()) {
            out.extend(
                vec![quote!(
                    let attr = discriminant_for.get("#field_name").unwrap();
                    self.#field_name.deserialize_with_discriminant(r, attr)?;
                )]
                .into_iter(),
            );
        } else {
            out.extend(
                vec![quote!(
                    self.#field_name.deserialize(r)?;
                )]
                .into_iter(),
            );
        }

        out
    });

    quote!(
        impl Deserialize for #name {
            fn deserialize<R: std::io::Read>(&mut self, r: &mut R) -> Result<()> {
                #map_declarations

                #(#deserializers) *

                Ok(())
            }
        }
    )
    .into()
}

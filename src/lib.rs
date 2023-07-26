use std::collections::HashMap;

use darling::{FromAttributes, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse, ItemStruct};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(ppproperly))]
#[darling(default)]
struct Args {
    discriminant_for: Option<DiscriminantArgs>,
    len_for: Option<LenArgs>,
}

#[derive(Debug, Default, FromMeta)]
struct DiscriminantArgs {
    field: String,
    data_type: String,
}

#[derive(Debug, Default, FromMeta)]
struct LenArgs {
    field: String,
    offset: u8,
    data_type: String,
}

#[proc_macro_derive(Serialize, attributes(ppproperly))]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let ast: ItemStruct = parse(item).unwrap();
    let name = ast.ident;

    let serializers = ast.fields.iter().map(|field| {
        let mut out = TokenStream2::new();

        let field_name = field.ident.as_ref().expect("should be a names struct");

        let args = Args::from_attributes(&field.attrs).unwrap();

        if let Some(attr) = args.discriminant_for {
            let field_ident = Ident::new(&attr.field, Span::call_site());

            out.extend(
                vec![quote!(
                    self.#field_ident.discriminant().serialize(w)?;
                )]
                .into_iter(),
            );
        }

        if let Some(attr) = args.len_for {
            let field_ident = Ident::new(&attr.field, Span::call_site());
            let offset = attr.offset;
            let data_type_ident = Ident::new(&attr.data_type, Span::call_site());

            out.extend(
                vec![quote!(
                    let n = #data_type_ident::try_from(self.#field_ident.len())?;
                    (n + #data_type_ident::from(#offset)).serialize(w)?;
                )]
                .into_iter(),
            );
        }

        out.extend(
            vec![quote!(
                self.#field_name.serialize(w)?;
            )]
            .into_iter(),
        );

        TokenTree::Group(Group::new(Delimiter::Brace, out))
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

    let mut discriminant_for = HashMap::new();
    let mut len_for = HashMap::new();

    let has_discriminant_annotations = ast.fields.iter().any(|field| {
        let args = Args::from_attributes(&field.attrs).unwrap();
        args.discriminant_for.is_some()
    });

    let has_len_annotations = ast.fields.iter().any(|field| {
        let args = Args::from_attributes(&field.attrs).unwrap();
        args.len_for.is_some()
    });

    let mut map_declarations = TokenStream2::new();

    if has_discriminant_annotations {
        map_declarations.extend(
            vec![quote!(
                let mut discriminant_for = std::collections::HashMap::new();
            )]
            .into_iter(),
        );
    }

    if has_len_annotations {
        map_declarations.extend(
            vec![quote!(
                let mut len_for = std::collections::HashMap::new();
            )]
            .into_iter(),
        );
    }

    let deserializers = ast.fields.iter().map(|field| {
        let mut out = TokenStream2::new();

        let field_name = field.ident.as_ref().expect("should be a names struct");
        let field_name_string = field_name.to_string();

        let args = Args::from_attributes(&field.attrs).unwrap();

        if let Some(attr) = args.discriminant_for {
            let field = attr.field;
            let data_type_ident = Ident::new(&attr.data_type, Span::call_site());

            out.extend(
                vec![quote!(
                    let mut discriminant = #data_type_ident::default();
                    discriminant.deserialize(r)?;

                    discriminant_for.insert(#field, discriminant);
                )]
                .into_iter(),
            );

            discriminant_for.insert(field, ());
        }

        if let Some(attr) = args.len_for {
            let field = attr.field;
            let offset = attr.offset;
            let data_type_ident = Ident::new(&attr.data_type, Span::call_site());

            out.extend(
                vec![quote!(
                    let mut len = #data_type_ident::default();
                    len.deserialize(r)?;

                    len_for.insert(#field, len - #data_type_ident::from(#offset));
                )]
                .into_iter(),
            );

            len_for.insert(field, ());
        }

        if len_for.contains_key(&field_name.to_string()) {
            out.extend(
                vec![quote!(
                    let r = &mut r.take(*len_for.get(#field_name_string).unwrap() as u64);
                )]
                .into_iter(),
            );
        }

        if discriminant_for.contains_key(&field_name.to_string()) {
            out.extend(
                vec![quote!(
                    let attr = discriminant_for.get(#field_name_string).unwrap();
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

        TokenTree::Group(Group::new(Delimiter::Brace, out))
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

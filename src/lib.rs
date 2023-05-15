use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, ItemStruct};

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let ast: ItemStruct = parse(item).unwrap();
    let name = ast.ident;

    quote!(
        impl Serialize for #name {
            fn serialize<W: Write>(&self, w: W) -> io::Result<()> {
                w.write_all(&[42])?;
                Ok(())
            }
        }
    )
    .into()
}

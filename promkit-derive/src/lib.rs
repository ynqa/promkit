extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Error, parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Promkit, attributes(readline))]
pub fn promkit_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match impl_promkit_derive(&ast) {
        Ok(token) => token.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

mod readline;

fn impl_promkit_derive(ast: &DeriveInput) -> Result<TokenStream, Error> {
    let fields = match &ast.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named,
            // tuple struct is like `struct Point(f32, f32);`
            syn::Fields::Unnamed(_) => {
                return Err(Error::new(ast.span(), "Not support tuple structs"))
            }
            // unit struct is like `struct Marker;`
            syn::Fields::Unit => return Err(Error::new(ast.span(), "Not support unit structs")),
        },
        syn::Data::Enum(_) => return Err(Error::new(ast.span(), "Not support enums")),
        syn::Data::Union(_) => return Err(Error::new(ast.span(), "Not support unions")),
    };

    let mut fns = quote! {};

    for field in fields.iter() {
        for attr in field.attrs.iter() {
            #[allow(clippy::single_match)]
            match attr.path().get_ident().unwrap().to_string().as_str() {
                "readline" => {
                    let expr = readline::impl_promkit_per_field(field, attr)?;
                    fns = quote! {
                        #fns
                        #expr
                    };
                }
                _ => (),
            }
        }
    }

    let name = &ast.ident;
    Ok(quote! {
        impl #name {
            #fns
        }
    })
}

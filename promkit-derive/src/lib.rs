extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Error, parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Promkit, attributes(form))]
pub fn promkit_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match impl_promkit_derive(&ast) {
        Ok(token) => token.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

mod text_editor;

fn impl_promkit_derive(ast: &DeriveInput) -> Result<TokenStream, Error> {
    let fields = match &ast.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named,
            syn::Fields::Unnamed(_) => {
                return Err(Error::new(ast.span(), "Not support tuple structs"))
            }
            syn::Fields::Unit => return Err(Error::new(ast.span(), "Not support unit structs")),
        },
        syn::Data::Enum(_) => return Err(Error::new(ast.span(), "Not support enums")),
        syn::Data::Union(_) => return Err(Error::new(ast.span(), "Not support unions")),
    };

    let mut text_editor_states = Vec::new();
    let mut field_assignments = Vec::new();
    let mut field_types = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        for attr in field.attrs.iter() {
            #[allow(clippy::single_match)]
            match attr.path().get_ident().unwrap().to_string().as_str() {
                "form" => {
                    let state = text_editor::create_state(attr)?;
                    text_editor_states.push(state);

                    let field_ident = field.ident.as_ref().unwrap();
                    let idx_lit = syn::Index::from(idx);

                    match &field.ty {
                        syn::Type::Path(typ) => {
                            let last_segment = typ.path.segments.last().unwrap();
                            match last_segment.ident.to_string().as_str() {
                                "Option" => {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &last_segment.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(inner_type)) =
                                            args.args.first()
                                        {
                                            field_assignments.push(quote! {
                                                self.#field_ident = results[#idx_lit].parse::<#inner_type>().ok();
                                            });
                                            field_types.push(quote! { Option<#inner_type> });
                                        }
                                    }
                                }
                                _ => {
                                    let ty = &field.ty;
                                    field_assignments.push(quote! {
                                        self.#field_ident = results[#idx_lit].parse::<#ty>()?;
                                    });
                                    field_types.push(quote! { #ty });
                                }
                            }
                        }
                        ty => {
                            return Err(Error::new(
                                ty.span(),
                                format!(
                                    "Support only Path for field type but got {}",
                                    ty.to_token_stream(),
                                ),
                            ))
                        }
                    }
                }
                _ => (),
            }
        }
    }

    let name = &ast.ident;
    let combined_states = quote! {
        vec![
            #(#text_editor_states),*
        ]
    };

    Ok(quote! {
        impl #name {
            pub async fn build(&mut self) -> Result<(), Box<dyn std::error::Error>> {
                use promkit::Prompt;

                let states = #combined_states;
                let mut form = promkit::preset::form::Form::new(states);
                let results = form.run().await?;

                #(#field_assignments)*

                Ok(())
            }
        }
    })
}

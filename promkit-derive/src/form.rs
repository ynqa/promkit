use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Error, punctuated::Punctuated, spanned::Spanned, Meta, MetaList, MetaNameValue, Token,
};

pub fn impl_promkit_per_field(
    field: &syn::Field,
    attr: &syn::Attribute,
) -> Result<TokenStream, Error> {
    let readline_preset: TokenStream = match &attr.meta {
        Meta::List(list) => {
            let results = [parse_default_meta(list), parse_kvs_meta(list)];
            let errors: Vec<Error> = results
                .iter()
                .filter_map(|r| r.as_ref().err().cloned())
                .collect();

            if errors.len() == results.len() {
                let error_messages = errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::new(
                    list.span(),
                    format!("Errors: {}", error_messages),
                ))
            } else {
                results
                    .into_iter()
                    .find_map(Result::ok)
                    .ok_or_else(|| Error::new(list.span(), "Unexpected error"))
            }
        }?,
        others => {
            return Err(Error::new(
                others.span(),
                format!(
                    "Support only readline(default), or readline(key=value, ...), but got {}",
                    others.to_token_stream()
                ),
            ))
        }
    };

    let field_ident = field.ident.as_ref().unwrap();
    let preset_fn = syn::Ident::new(&format!("readline_{}", field_ident), field_ident.span());

    match &field.ty {
        syn::Type::Path(typ) => {
            let last_segment = typ.path.segments.last().unwrap();
            match last_segment.ident.to_string().as_str() {
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            return Ok(quote! {
                                pub fn #preset_fn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
                                    let value_str = #readline_preset?;
                                    let parsed_value = value_str.parse::<#inner_type>()
                                        .map_or_else(|_| None, Some);
                                    self.#field_ident = parsed_value;
                                    Ok(())
                                }
                            });
                        }
                    }
                    Err(Error::new(
                        last_segment.span(),
                        format!("Support Option<T> but got {}", typ.to_token_stream()),
                    ))
                }
                _ => {
                    let ty = typ.to_token_stream();
                    Ok(quote! {
                        pub fn #preset_fn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
                            let value_str = #readline_preset?;
                            let parsed_value = value_str.parse::<#ty>()?;
                            self.#field_ident = parsed_value;
                            Ok(())
                        }
                    })
                }
            }
        }
        ty => Err(Error::new(
            ty.span(),
            format!(
                "Support only Path for field type but got {}",
                ty.to_token_stream()
            ),
        )),
    }
}

fn parse_default_meta(list: &MetaList) -> Result<TokenStream, Error> {
    match list.tokens.to_string().as_str() {
        "default" => Ok(quote! {
            promkit::preset::readline::Readline::default()
                .prompt()?
                .run()
        }),
        others => Err(Error::new(
            list.span(),
            format!("Support readline(default) but got {}", others),
        )),
    }
}

fn parse_kvs_meta(list: &MetaList) -> Result<TokenStream, Error> {
    let mut ret = quote! {
        promkit::preset::readline::Readline::default()
    };

    list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)
        .map_err(|e| {
            Error::new(
                list.span(),
                format!(
                    "Support readline(key=value, ...) but got {}, caused error: {}",
                    list.tokens, e
                ),
            )
        })?
        .into_iter()
        .for_each(
            |entry| match entry.path.get_ident().unwrap().to_string().as_str() {
                "prefix" => {
                    let expr = entry.value;
                    ret = quote! {
                        #ret
                        .prefix(format!("{} ", #expr))
                    };
                }
                "mask" => {
                    let expr = entry.value;
                    ret = quote! {
                        #ret
                        .mask(#expr)
                    };
                }
                "prefix_style" => {
                    let expr = entry.value;
                    ret = quote! {
                        #ret
                        .prefix_style(#expr)
                    };
                }
                "active_char_style" => {
                    let expr = entry.value;
                    ret = quote! {
                        #ret
                        .active_char_style(#expr)
                    };
                }
                "inactive_char_style" => {
                    let expr = entry.value;
                    ret = quote! {
                        #ret
                        .inactive_char_style(#expr)
                    };
                }
                _ => (),
            },
        );

    ret = quote! {
        #ret
        .prompt()?
        .run()
    };

    Ok(ret)
}

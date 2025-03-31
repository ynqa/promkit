use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Error, punctuated::Punctuated, spanned::Spanned, Meta, MetaNameValue, Token};

pub fn create_state(attr: &syn::Attribute) -> Result<TokenStream, Error> {
    let mut prefix = quote! { String::from("❯❯ ") };
    let mut prefix_style = quote! {
        promkit::crossterm::style::ContentStyle {
            attributes: promkit::crossterm::style::Attributes::from(promkit::crossterm::style::Attribute::Bold),
            ..Default::default()
        }
    };
    let mut active_char_style = quote! {
        promkit::crossterm::style::ContentStyle {
            background_color: Some(promkit::crossterm::style::Color::DarkCyan),
            ..Default::default()
        }
    };
    let mut inactive_char_style = quote! {
        promkit::crossterm::style::ContentStyle::default()
    };
    let mut mask = quote! { None::<char> };
    let mut edit_mode = quote! { promkit::promkit_widgets::text_editor::Mode::default() };
    let mut word_break_chars = quote! { std::collections::HashSet::from([' ']) };

    match &attr.meta {
        Meta::List(list) => {
            if list.tokens.to_string() != "default" {
                list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)
                    .map_err(|e| {
                        Error::new(
                            list.span(),
                            format!(
                                "Support form(key=value, ...) but got {}, caused error: {}",
                                list.tokens, e
                            ),
                        )
                    })?
                    .into_iter()
                    .for_each(
                        |entry| match entry.path.get_ident().unwrap().to_string().as_str() {
                            "label" => {
                                let expr = entry.value;
                                prefix = quote! { format!("{} ", #expr) };
                            }
                            "label_style" => {
                                let expr = entry.value;
                                prefix_style = quote! { #expr };
                            }
                            "active_char_style" => {
                                let expr = entry.value;
                                active_char_style = quote! { #expr };
                            }
                            "inactive_char_style" => {
                                let expr = entry.value;
                                inactive_char_style = quote! { #expr };
                            }
                            "mask" => {
                                let expr = entry.value;
                                mask = quote! { #expr };
                            }
                            "edit_mode" => {
                                let expr = entry.value;
                                edit_mode = quote! { #expr };
                            }
                            "word_break_chars" => {
                                let expr = entry.value;
                                word_break_chars = quote! { #expr };
                            }
                            _ => (),
                        },
                    );
            }
        }
        others => {
            return Err(Error::new(
                others.span(),
                format!(
                    "Support only form, form(default), or form(key=value, ...), but got {}",
                    others.to_token_stream()
                ),
            ))
        }
    };

    Ok(quote! {
        promkit::promkit_widgets::text_editor::State {
            texteditor: Default::default(),
            history: Default::default(),
            prefix: #prefix,
            prefix_style: #prefix_style,
            active_char_style: #active_char_style,
            inactive_char_style: #inactive_char_style,
            mask: #mask,
            edit_mode: #edit_mode,
            word_break_chars: #word_break_chars,
            lines: Default::default()
        }
    })
}

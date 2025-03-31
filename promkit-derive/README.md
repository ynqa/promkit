# promkit-derive

Macro for promkit derives.

## Convenient macros for form inputs

`promkit-derive` provides a Derive macro that simplifies interactive form input.
By applying the `#[derive(Promkit)]` attribute to a struct,
you can automatically generate interactive forms that populate your struct fields with user input.

## Features

- **Automated Form Generation**: The `#[derive(Promkit)]` macro automatically creates
  interactive form fields based on your struct definition
- **Type-Safe Conversion**: Form inputs are automatically converted to
  the appropriate types for your struct fields
- **Customizable**: Fine-tune the appearance and behavior of your forms with labels,
  styles, input modes, and more

## Usage Example

```rust
use promkit::crossterm::style::{Color, ContentStyle};
use promkit_derive::Promkit;

#[derive(Default, Debug, Promkit)]
struct Profile {
    #[form(
        label = "What is your name?",
        label_style = ContentStyle {
            foreground_color: Some(Color::DarkCyan),
            ..Default::default()
        },
    )]
    name: String,

    #[form(default)]
    hobby: Option<String>,

    #[form(label = "How old are you?", ignore_invalid_attr = "nothing")]
    age: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ret = Profile::default();
    ret.build()?;
    dbg!(ret);
    Ok(())
}
```

## Attribute Options

The `#[form(...)]` attribute supports the following options:

- `label`: Text label for the form field
- `label_style`: Style settings for the label
- `active_char_style`: Style for active characters
- `inactive_char_style`: Style for inactive characters
- `mask`: Masking character for password inputs
- `edit_mode`: Edit mode settings
- `word_break_chars`: Word break character settings
- `default`: Use default settings (no options)

## Supported Types

- Basic types (`String`, `usize`, `i32`, etc.)
- `Option<T>` types (becomes `None` if input is invalid)

## How It Works

The `#[derive(Promkit)]` macro implements a `build()` method on your struct that:

1. Generates text input fields for each field with a `#[form]` attribute
2. Displays an interactive form to collect user input
3. Converts the input values to the appropriate types and assigns them to your struct fields

This allows you to create interactive form inputs with minimal code,
simply by defining your struct and its fields.

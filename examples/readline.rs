use std::iter::FromIterator;

use anyhow::Result;

use promkit::{
    crossterm::style::Color,
    item_box::ItemBox,
    style::ContentStyleBuilder,
    suggest::Suggest,
    widgets::{ItemPickerBuilder, Mode, TextBuilder, TextEditorBuilder},
    Prompt,
};

fn main() -> Result<()> {
    let mut p = Prompt::new(vec![
        TextBuilder::new("Type Here")
            .style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Green)
                    .build(),
            )
            .make_state()?,
        TextEditorBuilder::default()
            .style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::DarkYellow)
                    .build(),
            )
            .cursor_style(
                ContentStyleBuilder::new()
                    .background_color(Color::DarkBlue)
                    .build(),
            )
            .label_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::DarkGreen)
                    .build(),
            )
            .suggest(Suggest::from_iter(["promkit", "ynqa"]))
            .make_state()?,
        TextEditorBuilder::default()
            .mode(Mode::Overwrite)
            .make_state()?,
        TextEditorBuilder::default()
            .mask('*')
            .cursor_style(
                ContentStyleBuilder::new()
                    .background_color(Color::Blue)
                    .build(),
            )
            .make_state()?,
        ItemPickerBuilder::new(ItemBox::from_iter(0..100))
            .cursor_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Magenta)
                    .build(),
            )
            .lines(5)
            .make_state()?,
        ItemPickerBuilder::new(ItemBox::from_iter(0..100))
            .cursor_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Magenta)
                    .build(),
            )
            .make_state()?,
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

use std::iter::FromIterator;

use anyhow::Result;

use promkit::{
    crossterm::style::Color,
    editor::{Mode, TextBuilder, TextEditorBuilder},
    style::ContentStyleBuilder,
    suggest::Suggest,
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
            .build()?,
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
            .build()?,
        TextEditorBuilder::default().mode(Mode::Overwrite).build()?,
        TextEditorBuilder::default().build()?,
        TextEditorBuilder::default().build()?,
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

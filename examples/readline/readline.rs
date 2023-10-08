use anyhow::Result;

use promkit::{
    crossterm::style::Color,
    editor::{TextBuilder, TextEditorBuilder},
    style::ContentStyleBuilder,
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
            .build()?,
        TextEditorBuilder::default().build()?,
        TextEditorBuilder::default().build()?,
        TextEditorBuilder::default().build()?,
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

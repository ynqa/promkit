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
        TextEditorBuilder::new()
            .label_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::DarkGreen)
                    .build(),
            )
            .build()?,
        TextEditorBuilder::new().build()?,
        TextEditorBuilder::new().build()?,
        TextEditorBuilder::new().build()?,
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

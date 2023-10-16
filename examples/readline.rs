use anyhow::Result;

use promkit::{crossterm::style::Color, preset::ReadlineBuilder, style::ContentStyleBuilder};

fn main() -> Result<()> {
    let mut p = ReadlineBuilder::default()
        .title(|builder| builder.text("hello"))
        .text_editor(|builder| {
            builder.cursor_style(
                ContentStyleBuilder::new()
                    .background_color(Color::DarkCyan)
                    .build(),
            )
        })
        .validator(
            |text| text.len() > 10,
            |text, builder| {
                builder
                    .text(format!("Length must be over 10 but got {}", text.len()))
                    .style(
                        ContentStyleBuilder::new()
                            .foreground_color(Color::DarkRed)
                            .build(),
                    )
            },
        )
        .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

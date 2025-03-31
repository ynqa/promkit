use promkit::{
    crossterm::style::{Color, ContentStyle},
    preset::form::Form,
    promkit_widgets::text_editor,
};

fn main() -> anyhow::Result<()> {
    let mut p = Form::new([
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkRed),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkGreen),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkBlue),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
    ])
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

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

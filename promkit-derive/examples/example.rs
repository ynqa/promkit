use promkit::{crossterm::style::Color, style::StyleBuilder};
use promkit_derive::Promkit;

#[derive(Default, Debug, Promkit)]
struct Profile {
    #[readline(
        prefix = "What is your name?",
        prefix_style = StyleBuilder::new().fgc(Color::DarkCyan).build(),
    )]
    name: String,

    #[readline(default)]
    hobby: Option<String>,

    #[readline(prefix = "How old are you?", ignore_invalid_attr = "nothing")]
    age: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ret = Profile::default();
    ret.readline_name()?;
    ret.readline_hobby()?;
    ret.readline_age()?;
    dbg!(ret);
    Ok(())
}

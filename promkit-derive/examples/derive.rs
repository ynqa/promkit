use promkit::{crossterm::style::Color, style::StyleBuilder};
use promkit_derive::Promkit;

#[derive(Default, Debug, Promkit)]
struct Profile {
    #[form(
        prefix = "What is your name?",
        prefix_style = StyleBuilder::new().fgc(Color::DarkCyan).build(),
    )]
    name: String,

    #[form(default)]
    hobby: Option<String>,

    #[form(prefix = "How old are you?", ignore_invalid_attr = "nothing")]
    age: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ret = Profile::default();
    ret.build()?;
    dbg!(ret);
    Ok(())
}

use promkit::{preset::readline::Readline, suggest::Suggest};
use std::io;

fn main() -> anyhow::Result<()> {
    let mut p = Readline::default()
        .title("Hi!")
        .enable_suggest(Suggest::from_iter([
            "apple",
            "applet",
            "application",
            "banana",
        ]))
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

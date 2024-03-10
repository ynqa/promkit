use promkit::{error::Result, preset::readline::Readline, suggest::Suggest};

fn main() -> Result {
    let mut p = Readline::default()
        .title("Feel free to fill in")
        .enable_suggest(Suggest::from_iter([
            "apple",
            "applet",
            "application",
            "banana",
        ]))
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

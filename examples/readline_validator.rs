use promkit::{error::Result, preset::readline::Readline};

fn main() -> Result {
    let mut p = Readline::default()
        .title("Feel free to fill in")
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

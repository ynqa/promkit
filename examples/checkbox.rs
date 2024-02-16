use promkit::{error::Result, preset::Checkbox};

fn main() -> Result {
    let mut p = Checkbox::new(0..100)
        .title("What number do you like?")
        .screen_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

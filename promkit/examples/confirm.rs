use promkit::{preset::confirm::Confirm, Result};

fn main() -> Result {
    let mut p = Confirm::new("Do you have a pet?").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

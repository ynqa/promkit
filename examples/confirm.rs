use promkit::{error::Result, preset::Confirm};

fn main() -> Result {
    let mut p = Confirm::new("Do you like programming?").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

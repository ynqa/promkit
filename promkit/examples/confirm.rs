use promkit::preset::confirm::Confirm;
use std::io;

fn main() -> anyhow::Result<()> {
    let mut p = Confirm::new("Do you have a pet?").prompt(io::stdout())?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

use promkit::{error::Result, preset::Readline};

fn main() -> Result {
    let mut p = Readline::default().title("Feel free to fill in").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

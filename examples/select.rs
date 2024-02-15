use promkit::{error::Result, preset::Select};

fn main() -> Result {
    let mut p = Select::new(0..100)
        .title("What number do you like?")
        .window_size(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

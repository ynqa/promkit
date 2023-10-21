use promkit::{error::Result, preset::Select};

fn main() -> Result {
    let mut p = Select::new(0..100)
        .title("What number do you like?")
        .lines(5)
        .prompt()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

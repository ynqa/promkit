use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

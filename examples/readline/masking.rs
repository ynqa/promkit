use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().mask('\0').build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

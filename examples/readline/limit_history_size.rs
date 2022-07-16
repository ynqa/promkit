use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().limit_history_size(3).build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

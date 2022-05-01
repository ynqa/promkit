use promptio::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().mask('\0').build()?;
    loop {
        let (line, exit_code) = p.run()?;
        if exit_code == 0 {
            println!("result: {:?}", line);
        } else {
            return Ok(());
        }
    }
}

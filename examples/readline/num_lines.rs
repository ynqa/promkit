use promptio::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().num_lines(1).build()?;
    loop {
        let (line, exit_code) = p.run()?;
        if exit_code == 0 {
            println!("result: {:?}", line);
        } else {
            return Ok(());
        }
    }
}

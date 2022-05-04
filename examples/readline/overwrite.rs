use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default()
        .edit_mode(readline::Mode::Overwrite)
        .build()?;
    loop {
        let (line, exit_code) = p.run()?;
        if exit_code == 0 {
            println!("result: {:?}", line);
        } else {
            return Ok(());
        }
    }
}

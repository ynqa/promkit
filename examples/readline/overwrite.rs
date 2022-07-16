use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default()
        .edit_mode(readline::Mode::Overwrite)
        .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

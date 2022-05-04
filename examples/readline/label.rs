use crossterm::style;

use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default()
        .label("??")
        .label_color(style::Color::DarkBlue)
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

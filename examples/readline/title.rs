use crossterm::style;

use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default()
        .title("Feel free to type here")
        .title_color(style::Color::DarkGreen)
        .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

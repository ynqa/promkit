use promkit::{build::Builder, crossterm::style, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default()
        .label("??")
        .label_color(style::Color::DarkBlue)
        .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}

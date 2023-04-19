use promkit::{build::Builder, crossterm::style, select, Result};

fn main() -> Result<()> {
    let mut p = select::Builder::new(0..100)
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .init_move_down_lines(1)
        .build()?;
    let line = p.run()?;
    println!("result: {:?}", line);
    Ok(())
}

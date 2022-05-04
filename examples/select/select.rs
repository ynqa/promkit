use crossterm::style;

use promkit::{
    build::Builder,
    edit::{Register, SelectBox},
    select, Result,
};

fn main() -> Result<()> {
    let mut selectbox = Box::new(SelectBox::default());
    selectbox.register_all((0..100).map(|v| v.to_string()).collect::<Vec<String>>());
    let mut p = select::Builder::default()
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .selectbox(selectbox)
        .build()?;
    let (line, exit_code) = p.run()?;
    if exit_code == 0 {
        println!("result: {:?}", line)
    }
    Ok(())
}

use promkit::{
    build::Builder, crossterm::style, register::Register, select, selectbox::SelectBox, Result,
};

fn main() -> Result<()> {
    let mut selectbox = SelectBox::default();
    selectbox.register_all((0..100).map(|v| v.to_string()).collect::<Vec<String>>());
    let mut p = select::Builder::default()
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .selectbox(selectbox)
        .init_move_down_lines(1)
        .build()?;
    let line = p.run()?;
    println!("result: {:?}", line);
    Ok(())
}

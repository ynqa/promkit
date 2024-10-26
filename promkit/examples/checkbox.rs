use promkit::preset::checkbox::Checkbox;
use std::io;

fn main() -> anyhow::Result<()> {
    let mut p = Checkbox::new(vec![
        "Apple",
        "Banana",
        "Orange",
        "Mango",
        "Strawberry",
        "Pineapple",
        "Grape",
        "Watermelon",
        "Kiwi",
        "Pear",
    ])
    .title("What are your favorite fruits?")
    .checkbox_lines(5)
    .prompt(io::stdout())?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

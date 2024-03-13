use promkit::{preset::checkbox::Checkbox, Result};

fn main() -> Result {
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
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

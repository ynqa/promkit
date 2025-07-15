use promkit::{preset::checkbox::Checkbox, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Checkbox::new(vec![
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
    .run()
    .await?;
    println!("result: {:?}", ret);
    Ok(())
}

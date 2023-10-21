use promkit::{error::Result, preset::QuerySelect};

fn main() -> Result {
    let mut p = QuerySelect::new(0..100, |text, items| -> Vec<String> {
        text.parse::<usize>()
            .map(|query| {
                items
                    .iter()
                    .filter(|num| query <= num.parse::<usize>().unwrap_or_default())
                    .map(|num| num.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or(items.clone())
    })
    .title("What number do you like?")
    .item_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

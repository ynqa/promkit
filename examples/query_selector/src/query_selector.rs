use promkit::{preset::query_selector::QuerySelector, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = QuerySelector::new(0..100, |text, items| -> Vec<String> {
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
    .listbox_lines(5)
    .run()
    .await?;
    println!("result: {:?}", ret);
    Ok(())
}

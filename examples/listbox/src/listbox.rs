use promkit::{preset::listbox::Listbox, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Listbox::new(0..100)
        .title("What number do you like?")
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

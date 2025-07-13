use promkit::preset::confirm::Confirm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Confirm::new_with_prefix("Do you have a pet?").run().await?;
    println!("result: {:?}", ret);
    Ok(())
}

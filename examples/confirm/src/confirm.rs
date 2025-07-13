use promkit::preset::confirm::Confirm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Confirm::try_default("Do you have a pet?")
        .await?
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

use promkit::preset::readline::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        match Readline::default().prompt().await {
            Ok(cmd) => {
                println!("result: {:?}", cmd);
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

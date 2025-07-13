use promkit::{preset::readline::Readline, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        match Readline::default().run().await {
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

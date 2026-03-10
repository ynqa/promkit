#[tokio::main]
async fn main() -> anyhow::Result<()> {
    zsh_pretend::run().await
}

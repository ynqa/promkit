use promkit::preset::password::Password;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Password::default()
        .title("Put your password")
        .validator(
            |text| 4 < text.len() && text.len() < 10,
            |text| format!("Length must be over 4 and within 10 but got {}", text.len()),
        )
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

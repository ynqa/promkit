use promkit::preset::text::Text;

fn main() -> anyhow::Result<()> {
    Text::new(std::fs::read_to_string("Cargo.toml")?)
        .prompt()?
        .run()?;
    Ok(())
}

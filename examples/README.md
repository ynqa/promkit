# Examples

## Readline

![readline](https://user-images.githubusercontent.com/6745370/175757317-94e75ddd-f968-43ba-8a3e-0e1e70191128.gif)

```rust
use promkit::{error::Result, preset::Readline};

fn main() -> Result {
    let mut p = Readline::default()
        .title("Feel free to fill in")
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .enable_history()
        .prompt()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
```

## Select

![select](https://user-images.githubusercontent.com/6745370/175757316-8499ace6-e520-465b-a3fe-671182015431.gif)

```rust
use promkit::{build::Builder, crossterm::style, select, Result};

fn main() -> Result<()> {
    let mut p = select::Builder::new(0..100)
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .build()?;
    let line = p.run()?;
    println!("result: {:?}", line);
    Ok(())
}
```

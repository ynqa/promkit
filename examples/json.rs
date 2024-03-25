use promkit::{json::JsonStream, preset::json::Json, serde_json::Deserializer, Result};

fn main() -> Result {
    let stream = JsonStream::new(
        Deserializer::from_str(
            r#"{
              "number": 9,
              "map": {
                "entry1": "first",
                "entry2": "second"
              },
              "list": [
                "abc",
                "def"
              ]
            }"#,
        )
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok),
        None,
    );

    let mut p = Json::new(stream)
        .title("JSON viewer")
        .json_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

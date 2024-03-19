use promkit::{json::JsonNode, preset::json::Json, Result};

fn main() -> Result {
    let mut p = Json::new(JsonNode::try_from(
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
    )?)
    .title("JSON viewer")
    .json_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

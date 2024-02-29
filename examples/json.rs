use promkit::{error::Result, json::JsonNode, preset::Json};

fn main() -> Result {
    let mut p = Json::new(JsonNode::try_from(
        r#"{
          "number": 1,
          "map": {
            "string1": "aaa",
            "string2": "bbb"
          },
          "list": [
            "abc",
            "def"
          ],
          "map_in_map": {
            "nested": {
              "leaf": "eof"
            }
          },
          "map_in_list": [
            {
              "map1": 1
            },
            {
              "map2": 2
            }
          ]
        }"#,
    )?)
    .title("JSON viewer")
    .json_lines(10)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

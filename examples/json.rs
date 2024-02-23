use promkit::{error::Result, json::JsonNode, preset::Json};

fn main() -> Result {
    let mut p = Json::new(JsonNode::new_from_str(
        r#"
    {
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
    .title("Select a directory or file")
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
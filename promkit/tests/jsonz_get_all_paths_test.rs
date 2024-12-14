#[cfg(test)]
mod get_all_paths {
    use std::{collections::HashSet, str::FromStr};

    use promkit::jsonz;
    use serde_json::Deserializer;

    #[test]
    fn test_get_all_paths() {
        let v = serde_json::Value::from_str(
            r#"
                {
                    "string": "value",
                    "number": 42,
                    "boolean": true,
                    "null": null,
                    "empty_object": {},
                    "empty_array": [],
                    "array": ["a", "b", "c"],
                    "nested": {
                        "field1": "value1",
                        "field2": {
                            "inner": "value2"
                        }
                    },
                    "mixed_array": [
                        {
                            "name": "first",
                            "values": [1, 2, 3]
                        },
                        {
                            "name": "second",
                            "values": []
                        }
                    ],
                    "complex": {
                        "data": [
                            {
                                "id": 1,
                                "items": [
                                    {"status": "active"},
                                    {"status": "inactive"}
                                ]
                            },
                            {
                                "id": 2,
                                "items": []
                            }
                        ]
                    }
                }
            "#,
        )
        .unwrap();

        let actual = jsonz::get_all_paths([&v]);
        let expected = HashSet::from_iter(
            [
                ".",
                ".array",
                ".array[0]",
                ".array[1]",
                ".array[2]",
                ".boolean",
                ".complex",
                ".complex.data",
                ".complex.data[0]",
                ".complex.data[0].id",
                ".complex.data[0].items",
                ".complex.data[0].items[0]",
                ".complex.data[0].items[0].status",
                ".complex.data[0].items[1]",
                ".complex.data[0].items[1].status",
                ".complex.data[1]",
                ".complex.data[1].id",
                ".complex.data[1].items",
                ".empty_array",
                ".empty_object",
                ".mixed_array",
                ".mixed_array[0]",
                ".mixed_array[0].name",
                ".mixed_array[0].values",
                ".mixed_array[0].values[0]",
                ".mixed_array[0].values[1]",
                ".mixed_array[0].values[2]",
                ".mixed_array[1]",
                ".mixed_array[1].name",
                ".mixed_array[1].values",
                ".nested",
                ".nested.field1",
                ".nested.field2",
                ".nested.field2.inner",
                ".null",
                ".number",
                ".string",
            ]
            .into_iter()
            .map(|e| e.to_string()),
        );

        assert_eq!(actual, expected, "Paths do not match expected values");
    }

    #[test]
    fn test_get_all_paths_for_jsonl() {
        let binding = Deserializer::from_str(
            r#"
                {"user": "alice", "age": 30, "hobbies": ["reading", "gaming"]}
                {"user": "bob", "age": 25, "settings": {"theme": "dark", "notifications": true}}
                {"user": "carol", "age": 28, "address": {"city": "Tokyo", "details": {"street": "Sakura", "number": 123}}}
            "#,
        )
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok)
        .collect::<Vec<_>>();

        let actual = jsonz::get_all_paths(binding.iter());
        let expected = HashSet::from_iter(
            [
                ".",
                ".address",
                ".address.city",
                ".address.details",
                ".address.details.number",
                ".address.details.street",
                ".age",
                ".hobbies",
                ".hobbies[0]",
                ".hobbies[1]",
                ".settings",
                ".settings.notifications",
                ".settings.theme",
                ".user",
            ]
            .into_iter()
            .map(|e| e.to_string()),
        );

        assert_eq!(actual, expected, "Paths do not match expected values");
    }
}

#[cfg(test)]
mod tests {
    use promkit::jsonz::*;

    fn create_test_rows() -> Vec<Row> {
        // テスト用のJSONデータ構造を作成:
        // {
        //   "array": [1, 2, {"key": "value"}],
        //   "object": {"a": 1, "b": 2}
        // }

        let mut rows = Vec::new();

        // ルートオブジェクト
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: 1,
                close_index: 13,
            },
            prev_sibling: None,
            next_sibling: None,
        });

        // "array": [...]
        rows.push(Row {
            parent: Some(0),
            depth: 1,
            key: Some("array".to_string()),
            value: Value::OpenContainer {
                container_type: ContainerType::Array,
                collapsed: false,
                first_child: 2,
                close_index: 6,
            },
            prev_sibling: None,
            next_sibling: Some(7),
        });

        // 1
        rows.push(Row {
            parent: Some(1),
            depth: 2,
            key: None,
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(3),
        });

        // 2
        rows.push(Row {
            parent: Some(1),
            depth: 2,
            key: None,
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(2),
            next_sibling: Some(4),
        });

        // {"key": "value"}
        rows.push(Row {
            parent: Some(1),
            depth: 2,
            key: None,
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: 5,
                close_index: 6,
            },
            prev_sibling: Some(3),
            next_sibling: None,
        });

        // "value"
        rows.push(Row {
            parent: Some(4),
            depth: 3,
            key: Some("key".to_string()),
            value: Value::String("value".to_string()),
            prev_sibling: None,
            next_sibling: None,
        });

        // } (内部オブジェクトの終わり)
        rows.push(Row {
            parent: Some(1),
            depth: 2,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: 5,
                open_index: 4,
            },
            prev_sibling: None,
            next_sibling: None,
        });

        // "object": {...}
        rows.push(Row {
            parent: Some(0),
            depth: 1,
            key: Some("object".to_string()),
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: 8,
                close_index: 12,
            },
            prev_sibling: Some(1),
            next_sibling: None,
        });

        // "a": 1
        rows.push(Row {
            parent: Some(7),
            depth: 2,
            key: Some("a".to_string()),
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(9),
        });

        // "b": 2
        rows.push(Row {
            parent: Some(7),
            depth: 2,
            key: Some("b".to_string()),
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(8),
            next_sibling: None,
        });

        // } (object の終わり)
        rows.push(Row {
            parent: Some(0),
            depth: 1,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: 9,
                open_index: 7,
            },
            prev_sibling: None,
            next_sibling: None,
        });

        // } (ルートオブジェクトの終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: 9,
                open_index: 0,
            },
            prev_sibling: None,
            next_sibling: None,
        });

        rows
    }

    fn create_test_jsonl_rows() -> Vec<Row> {
        // テスト用のJSONLデータ構造を作成:
        // {"id": 1, "name": "Alice"}
        // {"id": 2, "name": "Bob", "items": [1, 2, 3]}
        // {"id": 3, "name": "Charlie", "active": true}

        let mut rows = Vec::new();
        let mut current_index = 0;

        // 1行目: {"id": 1, "name": "Alice"}
        let first_obj_start = current_index;
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: first_obj_start + 1,
                close_index: first_obj_start + 3,
            },
            prev_sibling: None,
            next_sibling: Some(4),
        });
        current_index += 1;

        // "id": 1
        rows.push(Row {
            parent: Some(first_obj_start),
            depth: 1,
            key: Some("id".to_string()),
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(first_obj_start + 2),
        });
        current_index += 1;

        // "name": "Alice"
        rows.push(Row {
            parent: Some(first_obj_start),
            depth: 1,
            key: Some("name".to_string()),
            value: Value::String("Alice".to_string()),
            prev_sibling: Some(first_obj_start + 1),
            next_sibling: None,
        });
        current_index += 1;

        // } (1行目の終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: first_obj_start + 2,
                open_index: first_obj_start,
            },
            prev_sibling: None,
            next_sibling: Some(4),
        });
        current_index += 1;

        // 2行目: {"id": 2, "name": "Bob", "items": [1, 2, 3]}
        let second_obj_start = current_index;
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: second_obj_start + 1,
                close_index: second_obj_start + 8,
            },
            prev_sibling: Some(3),
            next_sibling: Some(13),
        });
        current_index += 1;

        // "id": 2
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
            key: Some("id".to_string()),
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: None,
            next_sibling: Some(second_obj_start + 2),
        });
        current_index += 1;

        // "name": "Bob"
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
            key: Some("name".to_string()),
            value: Value::String("Bob".to_string()),
            prev_sibling: Some(second_obj_start + 1),
            next_sibling: Some(second_obj_start + 3),
        });
        current_index += 1;

        // "items": [...]
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
            key: Some("items".to_string()),
            value: Value::OpenContainer {
                container_type: ContainerType::Array,
                collapsed: false,
                first_child: second_obj_start + 4,
                close_index: second_obj_start + 7,
            },
            prev_sibling: Some(second_obj_start + 2),
            next_sibling: None,
        });
        current_index += 1;

        // 1
        rows.push(Row {
            parent: Some(second_obj_start + 3),
            depth: 2,
            key: None,
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(second_obj_start + 5),
        });
        current_index += 1;

        // 2
        rows.push(Row {
            parent: Some(second_obj_start + 3),
            depth: 2,
            key: None,
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(second_obj_start + 4),
            next_sibling: Some(second_obj_start + 6),
        });
        current_index += 1;

        // 3
        rows.push(Row {
            parent: Some(second_obj_start + 3),
            depth: 2,
            key: None,
            value: Value::Number(serde_json::Number::from(3)),
            prev_sibling: Some(second_obj_start + 5),
            next_sibling: None,
        });
        current_index += 1;

        // ] (配列の終わり)
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Array,
                collapsed: false,
                last_child: second_obj_start + 6,
                open_index: second_obj_start + 3,
            },
            prev_sibling: None,
            next_sibling: None,
        });
        current_index += 1;

        // } (2行目の終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: second_obj_start + 7,
                open_index: second_obj_start,
            },
            prev_sibling: None,
            next_sibling: Some(13),
        });
        current_index += 1;

        // 3行目: {"id": 3, "name": "Charlie", "active": true}
        let third_obj_start = current_index;
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::OpenContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                first_child: third_obj_start + 1,
                close_index: third_obj_start + 4,
            },
            prev_sibling: Some(12),
            next_sibling: None,
        });
        current_index += 1;

        // "id": 3
        rows.push(Row {
            parent: Some(third_obj_start),
            depth: 1,
            key: Some("id".to_string()),
            value: Value::Number(serde_json::Number::from(3)),
            prev_sibling: None,
            next_sibling: Some(third_obj_start + 2),
        });
        current_index += 1;

        // "name": "Charlie"
        rows.push(Row {
            parent: Some(third_obj_start),
            depth: 1,
            key: Some("name".to_string()),
            value: Value::String("Charlie".to_string()),
            prev_sibling: Some(third_obj_start + 1),
            next_sibling: Some(third_obj_start + 3),
        });
        current_index += 1;

        // "active": true
        rows.push(Row {
            parent: Some(third_obj_start),
            depth: 1,
            key: Some("active".to_string()),
            value: Value::Boolean(true),
            prev_sibling: Some(third_obj_start + 2),
            next_sibling: None,
        });
        current_index += 1;

        // } (3行目の終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
            key: None,
            value: Value::CloseContainer {
                container_type: ContainerType::Object,
                collapsed: false,
                last_child: third_obj_start + 3,
                open_index: third_obj_start,
            },
            prev_sibling: None,
            next_sibling: None,
        });

        rows
    }

    #[test]
    fn test_up_movement() {
        let rows = create_test_rows();

        // 数値から親のコンテナへ
        assert_eq!(rows.up(2), Some(1));
        assert!(matches!(&rows[2].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[2].key, None); // 配列の要素なのでキーはない

        // 兄弟間の移動
        assert_eq!(rows.up(3), Some(2));
        assert!(matches!(&rows[3].value, Value::Number(n) if n == &serde_json::Number::from(2)));
        assert_eq!(rows[3].key, None); // 配列の要素なのでキーはない

        // ルートの上には何もない
        assert_eq!(rows.up(0), None);
    }

    #[test]
    fn test_down_movement() {
        let rows = create_test_rows();

        // コンテナから最初の子へ
        assert_eq!(rows.down(1), Some(2));
        assert!(matches!(&rows[2].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[2].key, None); // 配列の要素なのでキーはない

        // 兄弟間の移動
        assert_eq!(rows.down(2), Some(3));
        assert!(matches!(&rows[3].value, Value::Number(n) if n == &serde_json::Number::from(2)));
        assert_eq!(rows[3].key, None); // 配列の要素なのでキーはない

        // 末尾の次は何もない
        assert_eq!(rows.down(13), None);
    }

    #[test]
    fn test_collapse_expand() {
        let mut rows = create_test_rows();

        // コンテナの折りたたみ
        assert!(rows.collapse(1));
        assert!(!rows.collapse(1)); // 2回目は失敗

        // 展開
        assert!(rows.expand(1));
        assert!(!rows.expand(1)); // 2回目は失敗

        // 非コンテナは操作不可
        assert!(!rows.collapse(2));
        assert!(!rows.expand(2));
    }

    #[test]
    fn test_get_visible_rows() {
        let mut rows = create_test_rows();

        // すべて展開されている状態
        let visible = rows.get_visible_rows(0, 5);
        assert_eq!(visible, vec![0, 1, 2, 3, 4]);
        assert!(matches!(&rows[2].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[2].key, None); // 配列の要素なのでキーはない

        // 配列を折りたたむ
        rows.collapse(1);
        let visible = rows.get_visible_rows(0, 5);
        assert_eq!(visible, vec![0, 1, 7, 8, 9]);
        assert!(matches!(&rows[8].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[8].key, Some("a".to_string())); // オブジェクトのキー
    }

    #[test]
    fn test_jsonl_up_movement() {
        let rows = create_test_jsonl_rows();

        // 2行目の配列内の要素から上へ
        assert_eq!(rows.up(9), Some(8)); // 2から1へ
        assert!(matches!(&rows[9].value, Value::Number(n) if n == &serde_json::Number::from(2)));
        assert_eq!(rows[9].key, None); // 配列の要素なのでキーはない

        assert_eq!(rows.up(8), Some(7)); // 1から配列の開始へ
        assert!(matches!(&rows[8].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[8].key, None); // 配列の要素なのでキーはない

        // 行をまたいだ移動
        assert_eq!(rows.up(4), Some(3)); // 2行目の開始から1行目の終わりへ
        assert_eq!(rows.up(13), Some(12)); // 3行目の開始から2行目の終わりへ
    }

    #[test]
    fn test_jsonl_down_movement() {
        let rows = create_test_jsonl_rows();

        // 1行目から2行目へ
        assert_eq!(rows.down(3), Some(4));

        // 2行目の配列内の移動
        assert_eq!(rows.down(8), Some(9)); // 1から2へ
        assert!(matches!(&rows[8].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[8].key, None); // 配列の要素なのでキーはない

        assert_eq!(rows.down(9), Some(10)); // 2から3へ
        assert!(matches!(&rows[10].value, Value::Number(n) if n == &serde_json::Number::from(3)));
        assert_eq!(rows[10].key, None); // 配列の要素なのでキーはない

        // 2行目から3行目へ
        assert_eq!(rows.down(12), Some(13));
    }

    #[test]
    fn test_jsonl_collapse_expand() {
        let mut rows = create_test_jsonl_rows();

        // 2行目の配列を折りたたむ
        assert!(rows.collapse(7));
        assert!(!rows.collapse(7)); // 2回目は失敗

        // 2行目全体を折りたたむ
        assert!(rows.collapse(4));
        assert!(!rows.collapse(4)); // 2回目は失敗

        // 展開
        assert!(rows.expand(4));
        assert!(rows.expand(7));
    }

    #[test]
    fn test_jsonl_get_visible_rows() {
        let mut rows = create_test_jsonl_rows();

        // すべて展開されている状態
        let visible = rows.get_visible_rows(0, 5);
        assert_eq!(visible, vec![0, 1, 2, 3, 4]);
        assert!(matches!(&rows[1].value, Value::Number(n) if n == &serde_json::Number::from(1)));
        assert_eq!(rows[1].key, Some("id".to_string())); // オブジェクトのキー
        assert!(matches!(&rows[2].value, Value::String(s) if s == "Alice"));
        assert_eq!(rows[2].key, Some("name".to_string())); // オブジェクトのキー

        // 2行目の配列を折りたたむ
        rows.collapse(7);
        let visible = rows.get_visible_rows(0, 10);
        assert!(visible.contains(&7)); // 配列自体は表示
        assert!(!visible.contains(&8)); // 配列の中身は非表示

        // 2行目全体を折りたたむ
        rows.collapse(4);
        let visible = rows.get_visible_rows(0, 10);
        assert!(visible.contains(&4)); // 2行目自体は表示
        assert!(!visible.contains(&5)); // 2行目の中身は非表示
        assert!(visible.contains(&13)); // 3行目は表示
    }

    #[test]
    fn test_key_value_pairs() {
        let rows = create_test_jsonl_rows();

        // オブジェクトのキーと値のペアをテスト
        assert_eq!(rows[1].key, Some("id".to_string()));
        assert!(matches!(&rows[1].value, Value::Number(n) if n == &serde_json::Number::from(1)));

        assert_eq!(rows[2].key, Some("name".to_string()));
        assert!(matches!(&rows[2].value, Value::String(s) if s == "Alice"));

        // 配列の要素にはキーがないことをテスト
        assert_eq!(rows[9].key, None);
        assert!(matches!(&rows[9].value, Value::Number(n) if n == &serde_json::Number::from(2)));

        // 真偽値のキーと値のペアをテスト
        assert_eq!(rows[16].key, Some("active".to_string()));
        assert!(matches!(&rows[16].value, Value::Boolean(b) if *b == true));
    }
}

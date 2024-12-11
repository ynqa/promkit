use winnow::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(serde_json::Number),
    String(String),
    OpenContainer {
        container_type: ContainerType,
        collapsed: bool,
        first_child: usize,
        close_index: usize,
    },
    CloseContainer {
        container_type: ContainerType,
        collapsed: bool,
        last_child: usize,
        open_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerType {
    Array,
    Object,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub parent: Option<usize>,
    pub depth: usize,
    pub value: Value,
    pub prev_sibling: Option<usize>,
    pub next_sibling: Option<usize>,
}

pub struct JsonParser<'a> {
    input: &'a str,
    parents: Vec<usize>,
    rows: Vec<Row>,
}

impl<'a> JsonParser<'a> {}

pub trait RowOperations {
    fn up(&self, current: usize) -> Option<usize>;
    fn down(&self, current: usize) -> Option<usize>;
    fn collapse(&mut self, current: usize) -> bool;
    fn expand(&mut self, current: usize) -> bool;
    fn get_visible_rows(&self, start: usize, count: usize) -> Vec<usize>;
    fn is_visible(&self, index: usize) -> bool;
    fn next_visible(&self, current: usize) -> Option<usize>;
    fn find_next_sibling(&self, current: usize) -> Option<usize>;
}

impl RowOperations for Vec<Row> {
    fn up(&self, current: usize) -> Option<usize> {
        if current >= self.len() {
            return None;
        }

        // まず前の兄弟を確認
        if let Some(prev) = self[current].prev_sibling {
            // 前の兄弟がコンテナで折りたたまれていない場合、最後の子孫を返す
            match self[prev].value {
                Value::OpenContainer {
                    collapsed: false,
                    close_index,
                    ..
                } => {
                    let last_child =
                        if let Value::CloseContainer { last_child, .. } = self[close_index].value {
                            last_child
                        } else {
                            close_index
                        };
                    Some(last_child)
                }
                _ => Some(prev),
            }
        } else {
            // 親を返す
            self[current].parent
        }
    }

    fn down(&self, current: usize) -> Option<usize> {
        if current >= self.len() {
            return None;
        }

        // まずコンテナの場合を確認
        match self[current].value {
            Value::OpenContainer {
                collapsed: false,
                first_child,
                ..
            } => Some(first_child),
            Value::CloseContainer { open_index, .. } => {
                // CloseContainerの場合は、親の次の兄弟を探す
                self[open_index].next_sibling
            }
            _ => {
                // 次の兄弟を確認
                if let Some(next) = self[current].next_sibling {
                    Some(next)
                } else if let Some(parent) = self[current].parent {
                    // 親がCloseContainerの場合、その親の次の兄弟を探す
                    match self[parent].value {
                        Value::CloseContainer { open_index, .. } => self[open_index].next_sibling,
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    fn collapse(&mut self, current: usize) -> bool {
        if current >= self.len() {
            return false;
        }

        match self[current].value {
            Value::OpenContainer {
                ref mut collapsed, ..
            } => {
                if *collapsed {
                    false
                } else {
                    *collapsed = true;
                    true
                }
            }
            _ => false,
        }
    }

    fn expand(&mut self, current: usize) -> bool {
        if current >= self.len() {
            return false;
        }

        match self[current].value {
            Value::OpenContainer {
                ref mut collapsed, ..
            } => {
                if !*collapsed {
                    false
                } else {
                    *collapsed = false;
                    true
                }
            }
            _ => false,
        }
    }

    fn is_visible(&self, index: usize) -> bool {
        if index >= self.len() {
            return false;
        }

        let mut current = index;
        while let Some(parent) = self[current].parent {
            if let Value::OpenContainer {
                collapsed: true, ..
            } = self[parent].value
            {
                return false;
            }
            current = parent;
        }
        true
    }

    fn find_next_sibling(&self, current: usize) -> Option<usize> {
        if current >= self.len() {
            return None;
        }

        // 直接の次の兄弟を確認
        if let Some(next) = self[current].next_sibling {
            return Some(next);
        }

        // 親をたどって次の兄弟を探す
        let mut curr = current;
        while let Some(parent) = self[curr].parent {
            if let Some(next) = self[parent].next_sibling {
                return Some(next);
            }
            curr = parent;
        }
        None
    }

    fn next_visible(&self, current: usize) -> Option<usize> {
        if current >= self.len() {
            return None;
        }

        match self[current].value {
            Value::OpenContainer {
                collapsed: true,
                close_index,
                ..
            } => {
                // 折りたたまれている場合は次の兄弟へ
                self.find_next_sibling(current)
            }
            Value::OpenContainer { first_child, .. } => {
                // 展開されている場合は最初の子へ
                Some(first_child)
            }
            Value::CloseContainer { open_index, .. } => {
                // 閉じタグの次の兄弟を探す
                self.find_next_sibling(open_index)
            }
            _ => {
                // 次の兄弟があればそれを返す
                if let Some(next) = self[current].next_sibling {
                    Some(next)
                } else if let Some(parent) = self[current].parent {
                    // 親がOpenContainerならその閉じタグへ
                    match self[parent].value {
                        Value::OpenContainer { close_index, .. } => Some(close_index),
                        _ => self.find_next_sibling(parent),
                    }
                } else {
                    None
                }
            }
        }
    }

    fn get_visible_rows(&self, start: usize, count: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut current = Some(start);

        while let Some(idx) = current {
            if result.len() >= count {
                break;
            }

            result.push(idx);

            match self[idx].value {
                Value::OpenContainer {
                    collapsed: true, ..
                } => {
                    // 折りたたまれている場合は次の兄弟へ
                    current = self.find_next_sibling(idx);
                }
                _ => {
                    current = self.next_visible(idx);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(3),
        });

        // 2
        rows.push(Row {
            parent: Some(1),
            depth: 2,
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(2),
            next_sibling: Some(4),
        });

        // {"key": "value"}
        rows.push(Row {
            parent: Some(1),
            depth: 2,
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
            value: Value::String("value".to_string()),
            prev_sibling: None,
            next_sibling: None,
        });

        // } (内部オブジェクトの終わり)
        rows.push(Row {
            parent: Some(1),
            depth: 2,
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
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(9),
        });

        // "b": 2
        rows.push(Row {
            parent: Some(7),
            depth: 2,
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(8),
            next_sibling: None,
        });

        // } (object の終わり)
        rows.push(Row {
            parent: Some(0),
            depth: 1,
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
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(first_obj_start + 2),
        });
        current_index += 1;

        // "name": "Alice"
        rows.push(Row {
            parent: Some(first_obj_start),
            depth: 1,
            value: Value::String("Alice".to_string()),
            prev_sibling: Some(first_obj_start + 1),
            next_sibling: None,
        });
        current_index += 1;

        // } (1行目の終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
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
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: None,
            next_sibling: Some(second_obj_start + 2),
        });
        current_index += 1;

        // "name": "Bob"
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
            value: Value::String("Bob".to_string()),
            prev_sibling: Some(second_obj_start + 1),
            next_sibling: Some(second_obj_start + 3),
        });
        current_index += 1;

        // "items": [...]
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
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
            value: Value::Number(serde_json::Number::from(1)),
            prev_sibling: None,
            next_sibling: Some(second_obj_start + 5),
        });
        current_index += 1;

        // 2
        rows.push(Row {
            parent: Some(second_obj_start + 3),
            depth: 2,
            value: Value::Number(serde_json::Number::from(2)),
            prev_sibling: Some(second_obj_start + 4),
            next_sibling: Some(second_obj_start + 6),
        });
        current_index += 1;

        // 3
        rows.push(Row {
            parent: Some(second_obj_start + 3),
            depth: 2,
            value: Value::Number(serde_json::Number::from(3)),
            prev_sibling: Some(second_obj_start + 5),
            next_sibling: None,
        });
        current_index += 1;

        // ] (配列の終わり)
        rows.push(Row {
            parent: Some(second_obj_start),
            depth: 1,
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
            value: Value::Number(serde_json::Number::from(3)),
            prev_sibling: None,
            next_sibling: Some(third_obj_start + 2),
        });
        current_index += 1;

        // "name": "Charlie"
        rows.push(Row {
            parent: Some(third_obj_start),
            depth: 1,
            value: Value::String("Charlie".to_string()),
            prev_sibling: Some(third_obj_start + 1),
            next_sibling: Some(third_obj_start + 3),
        });
        current_index += 1;

        // "active": true
        rows.push(Row {
            parent: Some(third_obj_start),
            depth: 1,
            value: Value::Boolean(true),
            prev_sibling: Some(third_obj_start + 2),
            next_sibling: None,
        });
        current_index += 1;

        // } (3行目の終わり)
        rows.push(Row {
            parent: None,
            depth: 0,
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

        // 兄弟間の移動
        assert_eq!(rows.up(3), Some(2));

        // ルートの上には何もない
        assert_eq!(rows.up(0), None);
    }

    #[test]
    fn test_down_movement() {
        let rows = create_test_rows();

        // コンテナから最初の子へ
        assert_eq!(rows.down(1), Some(2));

        // 兄弟間の移動
        assert_eq!(rows.down(2), Some(3));

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
        assert_eq!(rows.get_visible_rows(0, 5), vec![0, 1, 2, 3, 4]);

        // 配列を折りたたむ
        rows.collapse(1);
        assert_eq!(rows.get_visible_rows(0, 5), vec![0, 1, 7, 8, 9]);
    }

    #[test]
    fn test_jsonl_up_movement() {
        let rows = create_test_jsonl_rows();

        // 2行目の配列内の要素から上へ
        assert_eq!(rows.up(9), Some(8)); // 2から1へ
        assert_eq!(rows.up(8), Some(7)); // 1から配列の開始へ

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
        assert_eq!(rows.down(9), Some(10)); // 2から3へ

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
        assert_eq!(rows.get_visible_rows(0, 5), vec![0, 1, 2, 3, 4]);

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
}

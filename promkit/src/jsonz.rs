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
    pub key: Option<String>,
    pub value: Value,
    pub prev_sibling: Option<usize>,
    pub next_sibling: Option<usize>,
}

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
            match &self[prev].value {
                Value::OpenContainer {
                    collapsed: false,
                    close_index,
                    ..
                } => {
                    let last_child = if let Value::CloseContainer { last_child, .. } =
                        &self[*close_index].value
                    {
                        *last_child
                    } else {
                        *close_index
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
        match &self[current].value {
            Value::OpenContainer {
                collapsed: false,
                first_child,
                ..
            } => Some(*first_child),
            Value::CloseContainer { open_index, .. } => {
                // CloseContainerの場合は、親の次の兄弟を探す
                self[*open_index].next_sibling
            }
            _ => {
                // 次の兄弟を確認
                if let Some(next) = self[current].next_sibling {
                    Some(next)
                } else if let Some(parent) = self[current].parent {
                    // 親がCloseContainerの場合、その親の次の兄弟を探す
                    match &self[parent].value {
                        Value::CloseContainer { open_index, .. } => self[*open_index].next_sibling,
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

        match &mut self[current].value {
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

        match &mut self[current].value {
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
            } = &self[parent].value
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

        match &self[current].value {
            Value::OpenContainer {
                collapsed: true,
                close_index: _,
                ..
            } => {
                // 折りたたまれている場合は次の兄弟へ
                self.find_next_sibling(current)
            }
            Value::OpenContainer { first_child, .. } => {
                // 展開されている場合は最初の子へ
                Some(*first_child)
            }
            Value::CloseContainer { open_index, .. } => {
                // 閉じタグの次の兄弟を探す
                self.find_next_sibling(*open_index)
            }
            _ => {
                // 次の兄弟があればそれを返す
                if let Some(next) = self[current].next_sibling {
                    Some(next)
                } else if let Some(parent) = self[current].parent {
                    // 親がOpenContainerならその閉じタグへ
                    match &self[parent].value {
                        Value::OpenContainer { close_index, .. } => Some(*close_index),
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

            match &self[idx].value {
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

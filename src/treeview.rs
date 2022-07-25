pub mod node {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::grapheme::Graphemes;

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Node {
        pub data: Graphemes,
        pub visible: bool,
        pub children: Vec<Rc<RefCell<Node>>>,
    }

    impl Node {
        pub fn flatten(&self) -> Vec<Graphemes> {
            let mut res = vec![];
            if self.visible {
                res.push(self.data.clone());
            }
            if !self.children.is_empty() {
                for child in self.children.iter() {
                    res.extend(child.borrow().flatten());
                }
            }
            res
        }

        fn find(&self, preorder: usize) -> Option<Node> {
            if preorder == 0 {
                return Some(self.clone());
            }

            if let Some(child) = self.children.iter().next() {
                return child.borrow().find(preorder - 1);
            }
            None
        }

        pub fn toggle(&mut self, preorder: usize) {
            if let Some(node) = self.find(preorder) {
                for child in &node.children {
                    let mut borrowed = child.borrow_mut();
                    borrowed.visible = !borrowed.visible;
                }
            }
        }
    }

    #[test]
    fn flatten() {
        let root = Node {
            data: Graphemes::from("/"),
            visible: true,
            children: vec![
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("a"),
                    visible: true,
                    children: vec![
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("aa"),
                            visible: true,
                            children: vec![],
                        })),
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("ab"),
                            visible: true,
                            children: vec![],
                        })),
                    ],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("b"),
                    visible: true,
                    children: vec![],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("c"),
                    visible: true,
                    children: vec![],
                })),
            ],
        };
        let expect = vec![
            Graphemes::from("/"),
            Graphemes::from("a"),
            Graphemes::from("aa"),
            Graphemes::from("ab"),
            Graphemes::from("b"),
            Graphemes::from("c"),
        ];
        assert_eq!(expect, root.flatten());
    }

    #[test]
    fn find() {
        let root = Node {
            data: Graphemes::from("/"),
            visible: true,
            children: vec![
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("a"),
                    visible: true,
                    children: vec![
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("aa"),
                            visible: true,
                            children: vec![],
                        })),
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("ab"),
                            visible: true,
                            children: vec![],
                        })),
                    ],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("b"),
                    visible: true,
                    children: vec![Rc::new(RefCell::new(Node {
                        data: Graphemes::from("bb"),
                        visible: true,
                        children: vec![],
                    }))],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("c"),
                    visible: true,
                    children: vec![],
                })),
            ],
        };
        assert_eq!(
            Some(Node {
                data: Graphemes::from("a"),
                visible: true,
                children: vec![
                    Rc::new(RefCell::new(Node {
                        data: Graphemes::from("aa"),
                        visible: true,
                        children: vec![],
                    })),
                    Rc::new(RefCell::new(Node {
                        data: Graphemes::from("ab"),
                        visible: true,
                        children: vec![],
                    })),
                ],
            }),
            root.find(1),
        );
    }

    #[test]
    fn toggle() {
        let mut root = Node {
            data: Graphemes::from("/"),
            visible: true,
            children: vec![
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("a"),
                    visible: true,
                    children: vec![
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("aa"),
                            visible: true,
                            children: vec![],
                        })),
                        Rc::new(RefCell::new(Node {
                            data: Graphemes::from("ab"),
                            visible: true,
                            children: vec![],
                        })),
                    ],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("b"),
                    visible: true,
                    children: vec![Rc::new(RefCell::new(Node {
                        data: Graphemes::from("bb"),
                        visible: true,
                        children: vec![],
                    }))],
                })),
                Rc::new(RefCell::new(Node {
                    data: Graphemes::from("c"),
                    visible: true,
                    children: vec![],
                })),
            ],
        };
        let expect = vec![
            Graphemes::from("/"),
            Graphemes::from("a"),
            Graphemes::from("b"),
            Graphemes::from("bb"),
            Graphemes::from("c"),
        ];
        root.toggle(1);
        assert_eq!(expect, root.flatten());
    }
}

use std::cell::Cell;

use crate::grapheme::Graphemes;
use node::Node;

#[derive(Clone, Default)]
pub struct TreeView {
    pub root: Node,
    position: Cell<usize>,
}

impl TreeView {
    pub fn position(&self) -> usize {
        self.position.get()
    }

    pub fn prev(&self) -> bool {
        if 0 < self.position.get() {
            self.position.set(self.position.get() - 1);
            return true;
        }
        false
    }

    pub fn next(&self) -> bool {
        let limit = self.root.flatten().len() - 1;
        if self.position.get() < limit {
            self.position.set(self.position.get() + 1);
            return true;
        } else {
            self.position.set(limit);
        }
        false
    }

    pub fn flatten(&self) -> Vec<Graphemes> {
        self.flatten()
    }

    pub fn toggle(&mut self) {
        self.root.toggle(self.position())
    }
}

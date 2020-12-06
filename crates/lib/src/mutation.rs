use crate::Cache;
use crate::Node;
use crate::Red;
use crate::{Green, GreenKind};

pub trait GreenMutate {
    fn replace(&self, builder: &mut Cache, green: Green) -> Green;
    fn push_many(&self, builder: &mut Cache, green: Vec<Green>) -> Green;
    fn insert_many(&self, builder: &mut Cache, idx: usize, green: Vec<Green>) -> Green;
    fn remove(&self, builder: &mut Cache) -> Option<Green>;
}

impl GreenMutate for Red {
    fn remove(&self, builder: &mut Cache) -> Option<Green> {
        match self.kind().as_child() {
            Some((parent, index, _)) => {
                let mut children = parent.green().children().collect::<Vec<_>>();

                children.remove(index);

                let new_parent = builder.node(parent.name(), |_| children);
                Some(parent.replace(builder, new_parent))
            }
            None => None,
        }
    }
    fn replace(&self, builder: &mut Cache, green: Green) -> Green {
        match self.kind().as_child() {
            Some((parent, index, _)) => {
                let mut replacement = Some(green);
                let children = parent
                    .green()
                    .children()
                    .enumerate()
                    .map(|(i, child)| {
                        if i == index {
                            replacement.take().unwrap()
                        } else {
                            child
                        }
                    })
                    .collect();
                let new_parent = builder.replace_children(parent.green(), children);
                parent.replace(builder, new_parent)
            }
            None => green,
        }
    }

    fn push_many(&self, builder: &mut Cache, mut new_green: Vec<Green>) -> Green {
        assert!(self.is_node());

        let mut children = self.green().children().collect::<Vec<_>>();
        children.append(&mut new_green);
        let new_red = builder.node(self.name(), |_| children);
        self.replace(builder, new_red)
    }

    fn insert_many(&self, builder: &mut Cache, idx: usize, green: Vec<Green>) -> Green {
        assert!(self.is_node());

        let mut children = self.green().children().collect::<Vec<_>>();

        let after = children.split_off(idx);

        let children = children
            .into_iter()
            .chain(green.into_iter())
            .chain(after.into_iter())
            .collect();

        let new_red = builder.node(self.name(), |_| children);
        self.replace(builder, new_red)
    }
}

pub fn replace_green(
    builder: &mut Cache,
    node: Green,
    f: impl Clone + Fn(&mut Cache, Green) -> Green,
) -> Green {
    let new = f(builder, node);
    if let GreenKind::Node(Node { children }) = &new.kind() {
        let new_children = children
            .iter()
            .map(|child| replace_green(builder, child.clone(), f.clone()))
            .collect::<Vec<_>>();

        let node = builder.node(new.name(), move |_| new_children);
        node
    } else {
        new
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn replace_second() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![builder.node("Add", |builder| {
                vec![
                    builder.token("number", "2"),
                    builder.token("op", "+"),
                    builder.token("number", "2"),
                ]
            })]
        });

        let root = Root::new(Red::root(tree)).unwrap();
        let op = root.op().unwrap();
        let second = op.right().unwrap();
        let new_token = builder.token("number", "3");
        let tree = second.0.replace(&mut builder, new_token);

        assert_eq!("2+3", tree.to_string());
    }

    #[test]
    fn push_extra() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![builder.node("Add", |builder| {
                vec![
                    builder.token("number", "2"),
                    builder.token("op", "+"),
                    builder.token("number", "2"),
                ]
            })]
        });

        let new_op = vec![
            builder.token("comma", ","),
            builder.node("Add", |builder| {
                vec![
                    builder.token("number", "3"),
                    builder.token("op", "-"),
                    builder.token("number", "2"),
                ]
            }),
        ];

        let root = Root::new(Red::root(tree)).unwrap();
        let tree = root.0.push_many(&mut builder, new_op);

        assert_eq!("2+2,3-2", tree.to_string());
    }

    #[test]
    fn insert_extra() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![
                builder.node("Add", |builder| {
                    vec![
                        builder.token("number", "1"),
                        builder.token("op", "+"),
                        builder.token("number", "2"),
                    ]
                }),
                builder.token("comma", ","),
                builder.node("Add", |builder| {
                    vec![
                        builder.token("number", "5"),
                        builder.token("op", "-"),
                        builder.token("number", "6"),
                    ]
                }),
            ]
        });

        let new_op = vec![
            builder.token("comma", ","),
            builder.node("Add", |builder| {
                vec![
                    builder.token("number", "3"),
                    builder.token("op", "*"),
                    builder.token("number", "4"),
                ]
            }),
        ];

        let root = Root::new(Red::root(tree)).unwrap();
        let tree = root.0.insert_many(&mut builder, 1, new_op);

        assert_eq!("1+2,3*4,5-6", tree.to_string());
    }

    #[test]
    fn remove() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![
                builder.node("Add", |builder| {
                    vec![
                        builder.token("number", "1"),
                        builder.token("op", "+"),
                        builder.token("number", "2"),
                    ]
                }),
                builder.token("comma", ","),
                builder.node("Add", |builder| {
                    vec![
                        builder.token("number", "3"),
                        builder.token("op", "*"),
                        builder.token("number", "4"),
                    ]
                }),
                builder.token("comma", ","),
                builder.node("Add", |builder| {
                    vec![
                        builder.token("number", "5"),
                        builder.token("op", "-"),
                        builder.token("number", "6"),
                    ]
                }),
            ]
        });

        let root = Root::new(Red::root(tree)).unwrap();
        let mut ops = root.ops();
        let first = ops.nth(1).unwrap();
        let tree = first.0.remove(&mut builder).unwrap();
        let root = Root::new(Red::root(tree)).unwrap();
        let mut commas = root.commas();
        let first = commas.next().unwrap();
        let tree = first.0.remove(&mut builder).unwrap();

        assert_eq!("1+2,5-6", tree.to_string());
    }

    #[test]
    fn replace_second_ws() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![builder.node("Add", |builder| {
                vec![
                    builder.with_trivia("number", "", "2", " "),
                    builder.with_trivia("op", "", "+", " "),
                    builder.token("number", "2"),
                ]
            })]
        });

        let root = Root::new(Red::root(tree)).unwrap();
        let op = root.op().unwrap();
        let second = op.right().unwrap();
        let new_token = builder.token("number", "3");
        let tree = second.0.replace(&mut builder, new_token);

        assert_eq!("2 + 3", tree.to_string());
    }

    #[test]
    fn remove_ws() {
        let mut builder = Cache::default();
        let tree = builder.node("Root", |builder| {
            vec![builder.node("Add", |builder| {
                vec![
                    builder.with_trivia("number", "", "2", " "),
                    builder.with_trivia("op", "", "+", " "),
                    builder.token("number", "2"),
                ]
            })]
        });

        let tree = replace_green(&mut builder, tree, |builder, node| match &node.kind() {
            GreenKind::Token(Token { value, .. }) => builder.token(node.name(), value.clone()),
            _ => node,
        });

        let result = tree.to_string();

        assert_eq!("2+2", result);
    }

    #[derive(Debug)]
    struct Root(Red);
    impl Root {
        fn new(node: Red) -> Option<Self> {
            if node.name() != "Root" {
                return None;
            }
            node.green().as_node()?;
            Some(Self(node))
        }
    }

    impl Root {
        fn ops(&self) -> impl Iterator<Item = Op> + '_ {
            self.0.children().filter_map(Op::new)
        }

        fn op(&self) -> Option<Op> {
            self.ops().next()
        }

        fn commas(&self) -> impl Iterator<Item = Comma> + '_ {
            self.0.children().filter_map(Comma::new)
        }
    }

    #[derive(Debug)]
    struct Comma(Red);

    impl Comma {
        fn new(node: Red) -> Option<Self> {
            if node.name() != "comma" {
                return None;
            }
            node.green().as_token()?;
            Some(Self(node))
        }
    }

    #[derive(Debug)]
    struct Op(Red);

    impl Op {
        fn new(node: Red) -> Option<Self> {
            if node.name() != "Add" {
                return None;
            }
            node.green().as_node()?;
            Some(Self(node))
        }

        fn right(&self) -> Option<Number> {
            let child = self.0.children().nth(2)?;
            Number::new(child)
        }
    }

    #[derive(Debug)]
    struct Number(Red);

    impl Number {
        fn new(node: Red) -> Option<Self> {
            if node.name() != "number" {
                return None;
            }
            node.green().as_token()?;
            Some(Self(node))
        }
    }
}

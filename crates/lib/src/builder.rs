use crate::GreenData;
use crate::Node;
use crate::Token;
use crate::{Green, GreenKind, Name};
use std::sync::Arc;

use smol_str::SmolStr;

#[derive(Debug, Default)]
pub struct GreenBuilder {
    cache: Vec<Green>,
}

impl GreenBuilder {
    pub fn node(&mut self, name: Name, f: impl FnOnce(&mut Self) -> Vec<Green>) -> Green {
        let children = f(self);
        let size = children.iter().map(|g| g.size()).sum();
        self.add_node(GreenData {
            name,
            kind: GreenKind::Node(Node { children }),
            size,
        })
    }

    pub fn replace_children(&mut self, green: Green, children: Vec<Green>) -> Green {
        match green.kind() {
            GreenKind::Node(_) => {
                let size = children.iter().map(|g| g.size()).sum();
                self.add_node(GreenData {
                    name: green.name(),
                    size,
                    kind: GreenKind::Node(Node { children }),
                })
            }
            GreenKind::Alias(Some(child)) => {
                let alias_name = green.name();
                let child = self.replace_children(child.clone(), children);
                self.add_node(GreenData {
                    name: alias_name,
                    size: child.size(),
                    kind: GreenKind::Alias(Some(child)),
                })
            }
            GreenKind::Token(_) | GreenKind::Alias(None) => unreachable!(),
        }
    }

    pub fn alias<F, G>(&mut self, name: Name, f: F) -> Green
    where
        F: FnOnce(&mut Self) -> G,
        G: Into<Option<Green>>,
    {
        let child = f(self).into();
        let size = child.as_ref().map(|child| child.size()).unwrap_or_default();
        self.add_node(GreenData {
            name,
            size,
            kind: GreenKind::Alias(child),
        })
    }

    pub fn token(&mut self, name: Name, value: impl Into<SmolStr>) -> Green {
        self.with_trivia(name, "", value.into(), "")
    }

    pub fn with_trivia(
        &mut self,
        name: Name,
        prefix: impl Into<SmolStr>,
        value: impl Into<SmolStr>,
        postfix: impl Into<SmolStr>,
    ) -> Green {
        let prefix = prefix.into();
        let value = value.into();
        let postfix = postfix.into();
        self.add_node(GreenData {
            name,
            size: value.len() + prefix.len() + postfix.len(),
            kind: GreenKind::Token(Token {
                prefix,
                value,
                postfix,
            }),
        })
    }
}

#[cfg(test)]
impl GreenBuilder {
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl GreenBuilder {
    fn add_node(&mut self, node: GreenData) -> Green {
        match &node.kind {
            GreenKind::Node(Node { children }) if children.len() >= 5 => Green(Arc::new(node)),
            _ => {
                let node = Green(Arc::new(node));
                for cached in &self.cache {
                    let c: &Green = &*cached;
                    if c == &node {
                        return cached.clone();
                    }
                }
                self.cache.push(node.clone());
                node
            }
        }
    }
}

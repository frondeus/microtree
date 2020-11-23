use std::sync::Arc;

use crate::Green;
use crate::Name;

#[derive(Debug, Clone)]
pub struct RedData {
    kind: RedKind,
    green: Green,
}

#[derive(Debug, Clone)]
pub enum RedKind {
    Root,
    Child {
        parent: Red,
        index: usize,
        offset: usize,
    },
}

impl RedKind {
    pub fn as_child(&self) -> Option<(&Red, usize, usize)> {
        match self {
            Self::Child {
                parent,
                index,
                offset,
            } => Some((parent, *index, *offset)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Red(Arc<RedData>);

impl Red {
    pub fn root(green: Green) -> Self {
        Self(Arc::new(RedData {
            kind: RedKind::Root,
            green,
        }))
    }

    pub fn child(green: Green, parent: Red, index: usize, offset: usize) -> Self {
        Self(Arc::new(RedData {
            kind: RedKind::Child {
                parent,
                index,
                offset,
            },
            green,
        }))
    }

    pub fn green(&self) -> Green {
        self.0.green.clone()
    }

    pub fn is_alias(&self) -> bool {
        self.0.green.is_alias()
    }

    pub fn is_node(&self) -> bool {
        self.0.green.as_node().is_some()
    }

    pub fn kind(&self) -> &RedKind {
        &self.0.kind
    }

    pub fn name(&self) -> Name {
        self.green().name()
    }

    pub fn is(&self, name: Name) -> bool {
        self.green().is(name)
    }

    pub fn parent(&self) -> Option<Red> {
        let parent = self.0.kind.as_child()?.0;
        Some(parent.clone())
    }

    pub fn offset(&self) -> usize {
        self.0
            .kind
            .as_child()
            .map(|(_, _, o)| o)
            .unwrap_or_default()
    }

    pub fn children(&self) -> impl Iterator<Item = Red> + '_ {
        let parent = self.clone();
        let mut offset = self.offset();
        self.0
            .green
            .children()
            .enumerate()
            .map(move |(idx, green_child)| {
                let child_offset = offset;
                offset += green_child.size();
                Self::child(green_child, parent.clone(), idx, child_offset)
            })
    }
}

impl PartialEq for Red {
    fn eq(&self, other: &Self) -> bool {
        self.offset() == other.offset() && self.0.green == other.0.green
    }
}

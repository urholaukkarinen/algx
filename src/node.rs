
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct NodeId(usize);

impl Default for NodeId {
    fn default() -> Self {
        Self::invalid()
    }
}

impl NodeId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn invalid() -> Self {
        Self(usize::MAX)
    }

    pub fn is_valid(&self) -> bool {
        *self != Self::invalid()
    }

    pub fn value(&self) -> usize {
        assert!(self.is_valid());

        self.0
    }
}

#[derive(Default, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Node {
    pub(crate) left: NodeId,
    pub(crate) right: NodeId,
    pub(crate) up: NodeId,
    pub(crate) down: NodeId,
    pub(crate) header: NodeId,
    pub(crate) row: isize,
    pub(crate) col: usize,
}
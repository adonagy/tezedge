use storage::persistent::{Codec, SchemaError};
use serde::{Serialize, Deserialize};
use crate::LEVEL_BASE;
use std::collections::HashMap;

/// Structure for orientation in the list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeHeader {
    /// Level on which this node exists
    lane_level: usize,
    /// Position of node in lane
    node_index: usize,
}

impl NodeHeader {
    pub fn new(lane_level: usize, node_index: usize) -> Self {
        Self { lane_level, node_index }
    }

    pub fn next(&self) -> Self {
        Self {
            lane_level: self.lane_level,
            node_index: self.node_index + 1,
        }
    }

    pub fn lower(&self) -> Self {
        if self.lane_level == 0 {
            self.clone()
        } else {
            Self {
                lane_level: self.lane_level - 1,
                node_index: self.lower_index(),
            }
        }
    }

    pub fn base_index(&self) -> usize {
        if self.lane_level == 0 {
            self.node_index
        } else {
            ((self.node_index + 1) * LEVEL_BASE.pow(self.lane_level as u32)) - 1
        }
    }

    pub fn lower_index(&self) -> usize {
        if self.lane_level == 0 {
            self.node_index
        } else {
            ((self.node_index + 1) * LEVEL_BASE) - 1
        }
    }

    pub fn level(&self) -> usize {
        self.lane_level
    }

    pub fn index(&self) -> usize {
        self.node_index
    }
}

impl Codec for NodeHeader {
    fn decode(bytes: &[u8]) -> Result<Self, SchemaError> {
        bincode::deserialize(bytes)
            .map_err(|_| SchemaError::DecodeError)
    }

    fn encode(&self) -> Result<Vec<u8>, SchemaError> {
        bincode::serialize(self)
            .map_err(|_| SchemaError::EncodeError)
    }
}

/// Trait representing node value.
/// Value must be able to be recreated by merging, and then split by difference.
/// diff = A.diff(B);
/// A.merge(diff) == B
pub trait ListValue: Codec + Default {
    /// Merge two values into one, in-place
    fn merge(self, other: Self) -> Self;
    /// Create difference between two values.
    fn diff(self, other: Self) -> Self;
}

impl<K, V> ListValue for HashMap<K, V>
    where
        K: std::hash::Hash + Eq + Serialize + for<'a> Deserialize<'a> + Clone,
        V: Serialize + for<'a> Deserialize<'a> + Clone
{
    fn merge(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }

    fn diff(mut self, other: Self) -> Self {
        for (k, v) in other {
            if !self.contains_key(&k) {
                self.insert(k, v);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn header_next() {
        let original = NodeHeader::new(0, 0);
        let next = original.next();
        assert_eq!(original.lane_level, next.lane_level);
        assert_eq!(original.node_index + 1, next.node_index);
    }

    #[test]
    pub fn header_level() {
        let original = NodeHeader::new(0, 0);
        assert_eq!(original.level(), 0);
    }

    #[test]
    pub fn header_index() {
        let original = NodeHeader::new(0, 0);
        assert_eq!(original.index(), 0);
    }

    pub fn header_base_index() {
        let original = NodeHeader::new(0, 0);
        assert_eq!(original.base_index(), 0);
        let original = NodeHeader::new(1, 0);
        assert_eq!(original.base_index(), 7);
        let original = NodeHeader::new(2, 0);
        assert_eq!(original.base_index(), 63);
    }

    pub fn header_lower_index() {
        let original = NodeHeader::new(0, 0);
        assert_eq!(original.lower_index(), 0);
        let original = NodeHeader::new(1, 0);
        assert_eq!(original.lower_index(), 7);
        let original = NodeHeader::new(2, 0);
        assert_eq!(original.lower_index(), 7);
    }
}
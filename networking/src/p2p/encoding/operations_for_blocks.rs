// Copyright (c) SimpleStaking and Tezos-RS Contributors
// SPDX-License-Identifier: MIT

use std::mem::size_of;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use tezos_encoding::encoding::{Encoding, Field, HasEncoding, Tag, TagMap};
use tezos_encoding::hash::{BlockHash, Hash, HashEncoding, HashType};

use crate::p2p::binary_message::cache::{BinaryDataCache, CacheReader, CacheWriter, CachedData, NeverCache};
use crate::p2p::encoding::operation::Operation;

static DUMMY_BODY_CACHE: NeverCache = NeverCache;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OperationsForBlock {
    pub hash: BlockHash,
    pub validation_pass: i8,

    #[serde(skip_serializing)]
    pub body: BinaryDataCache,
}

impl OperationsForBlock {
    pub fn new(hash: &BlockHash, validation_pass: i8) -> Self {
        OperationsForBlock { hash: hash.clone(), validation_pass, body: Default::default() }
    }
}

impl HasEncoding for OperationsForBlock {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("hash", Encoding::Hash(HashEncoding::new(HashType::BlockHash))),
            Field::new("validation_pass", Encoding::Int8),
        ])
    }
}

impl CachedData for OperationsForBlock {
    #[inline]
    fn cache_reader(&self) -> & dyn CacheReader {
        &self.body
    }

    #[inline]
    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        Some(&mut self.body)
    }
}

// -----------------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OperationsForBlocksMessage {
    pub operations_for_block: OperationsForBlock,
    pub operation_hashes_path: Path,
    pub operations: Vec<Operation>,

    #[serde(skip_serializing)]
    pub body: BinaryDataCache
}

impl HasEncoding for OperationsForBlocksMessage {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("operations_for_block", OperationsForBlock::encoding()),
            Field::new("operation_hashes_path", path_encoding()),
            Field::new("operations", Encoding::list(Encoding::dynamic(Operation::encoding()))),
        ])
    }
}

impl CachedData for OperationsForBlocksMessage {
    #[inline]
    fn cache_reader(&self) -> & dyn CacheReader {
        &self.body
    }

    #[inline]
    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        Some(&mut self.body)
    }
}

// -----------------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PathRight {
    pub left: Hash,
    pub path: Path,

    #[serde(skip_serializing)]
    body: BinaryDataCache,
}

impl HasEncoding for PathRight {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("left", Encoding::Hash(HashEncoding::new(HashType::OperationListListHash))),
            Field::new("path", path_encoding()),
        ])
    }
}

impl CachedData for PathRight {
    #[inline]
    fn cache_reader(&self) -> & dyn CacheReader {
        &self.body
    }

    #[inline]
    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        Some(&mut self.body)
    }
}

// -----------------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PathLeft {
    pub path: Path,
    pub right: Hash,

    #[serde(skip_serializing)]
    body: BinaryDataCache,
}

impl HasEncoding for PathLeft {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("path", path_encoding()),
            Field::new("right", Encoding::Hash(HashEncoding::new(HashType::OperationListListHash))),
        ])
    }
}

impl CachedData for PathLeft {
    #[inline]
    fn cache_reader(&self) -> & dyn CacheReader {
        &self.body
    }

    #[inline]
    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        Some(&mut self.body)
    }
}

// -----------------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Path {
    Left(Box<PathLeft>),
    Right(Box<PathRight>),
    Op
}

fn path_encoding() -> Encoding {
    Encoding::Tags(
        size_of::<u8>(),
        TagMap::new(&[
            Tag::new(0xF0, "Left", Encoding::Lazy(Arc::new(PathLeft::encoding))),
            Tag::new(0x0F, "Right", Encoding::Lazy(Arc::new(PathRight::encoding))),
            Tag::new(0x00, "Op", Encoding::Unit),
        ])
    )
}

impl CachedData for Path {
    fn cache_reader(&self) -> & dyn CacheReader {
        &DUMMY_BODY_CACHE
    }

    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        None
    }
}

// -----------------------------------------------------------------------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct GetOperationsForBlocksMessage {
    pub get_operations_for_blocks: Vec<OperationsForBlock>,

    #[serde(skip_serializing)]
    body: BinaryDataCache,
}

impl GetOperationsForBlocksMessage {
    pub fn new(ops: Vec<OperationsForBlock>) -> Self {
        GetOperationsForBlocksMessage {
            get_operations_for_blocks: ops,
            body: Default::default()
        }
    }
}

impl HasEncoding for GetOperationsForBlocksMessage {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("get_operations_for_blocks", Encoding::dynamic(Encoding::list(OperationsForBlock::encoding()))),
        ])
    }
}

impl CachedData for GetOperationsForBlocksMessage {
    #[inline]
    fn cache_reader(&self) -> & dyn CacheReader {
        &self.body
    }

    #[inline]
    fn cache_writer(&mut self) -> Option<&mut dyn CacheWriter> {
        Some(&mut self.body)
    }
}

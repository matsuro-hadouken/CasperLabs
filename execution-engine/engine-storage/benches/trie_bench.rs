#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use casperlabs_engine_storage::trie::{Pointer, PointerBlock, Trie};
use engine_shared::{newtypes::Blake2bHash, stored_value::StoredValue};
use types::{
    account::PublicKey,
    bytesrepr::{FromBytes, ToBytes},
    CLValue, Key,
};

#[bench]
fn serialize_trie_leaf(b: &mut Bencher) {
    let leaf = Trie::Leaf {
        key: Key::Account(PublicKey::ed25519_from([0; 32])),
        value: StoredValue::CLValue(CLValue::from_t(42_i32).unwrap()),
    };
    b.iter(|| ToBytes::to_bytes(black_box(&leaf)));
}

#[bench]
fn deserialize_trie_leaf(b: &mut Bencher) {
    let leaf = Trie::Leaf {
        key: Key::Account(PublicKey::ed25519_from([0; 32])),
        value: StoredValue::CLValue(CLValue::from_t(42_i32).unwrap()),
    };
    let leaf_bytes = leaf.to_bytes().unwrap();
    b.iter(|| Trie::<Key, StoredValue>::from_bytes(black_box(&leaf_bytes)));
}

#[bench]
fn serialize_trie_node(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Node {
        pointer_block: Box::new(PointerBlock::default()),
    };
    b.iter(|| ToBytes::to_bytes(black_box(&node)));
}

#[bench]
fn deserialize_trie_node(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Node {
        pointer_block: Box::new(PointerBlock::default()),
    };
    let node_bytes = node.to_bytes().unwrap();

    b.iter(|| Trie::<Key, StoredValue>::from_bytes(black_box(&node_bytes)));
}

#[bench]
fn serialize_trie_node_pointer(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Extension {
        affix: (0..255).collect(),
        pointer: Pointer::NodePointer(Blake2bHash::new(&[0; 32])),
    };

    b.iter(|| ToBytes::to_bytes(black_box(&node)));
}

#[bench]
fn deserialize_trie_node_pointer(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Extension {
        affix: (0..255).collect(),
        pointer: Pointer::NodePointer(Blake2bHash::new(&[0; 32])),
    };
    let node_bytes = node.to_bytes().unwrap();

    b.iter(|| Trie::<Key, StoredValue>::from_bytes(black_box(&node_bytes)));
}

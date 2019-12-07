// PoC, needs refactoring

#[macro_use]
extern crate assert_json_diff;
extern crate reqwest;
extern crate serde;

#[derive(Debug)]
pub enum NodeType {
    Tezedge,
    Ocaml,
}

use chrono::DateTime;
use std::fmt;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test]
fn test_heads() {
    // should we use recursion?
    // TODO: test recursion

    let mut next_block = "BM9xFVaVv6mi7ckPbTgxEe7TStcfFmteJCpafUZcn75qi2wAHrC".to_string(); // 1000th

    while next_block != "" {
        let ocaml_json =
            get_block(NodeType::Ocaml, &next_block).expect("Failed to get block from ocaml");
        let tezedge_json =
            get_block(NodeType::Tezedge, &next_block).expect("Failed to get block from tezedge");
        let predecessor = ocaml_json["header"]["predecessor"]
            .to_string()
            .replace("\"", "");

        // NOTE: this will allways fail for now due to unimplemented properties in tezedge
        // print the asserted block, to know which one errored in case of an error
        println!("Checking: {}", &next_block);
        assert_json_eq!(tezedge_json, ocaml_json);

        // TODO: remove this line

        // debug: remove later
        // NOTE: cannot get genesis block from node
        if next_block == "BLockGenesisGenesisGenesisGenesisGenesisd1f7bcGMoXy" {
            println!("Genesis block reached and checked, breaking loop...");
            break;
        }
        next_block = predecessor;
    }
}

fn get_block(
    node: NodeType,
    block_id: &String,
) -> Result<serde_json::value::Value, serde_json::error::Error> {
    let url = match node {
        NodeType::Ocaml => format!(
            "http://ocaml-node-run:8732/chains/main/blocks/{}",
            block_id.replace("\"", "")
        ), // reference Ocaml node
        NodeType::Tezedge => format!(
            "http://tezedge-node-run:18732/chains/main/blocks/{}",
            block_id.replace("\"", "")
        ), // Tezedge node
    };

    let res = match reqwest::blocking::get(&url) {
        Ok(v) => v,
        Err(e) => panic!("Request for getting block failed: {}", e),
    };

    serde_json::from_str(&res.text().unwrap())
}

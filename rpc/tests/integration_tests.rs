#[macro_use]
extern crate assert_json_diff;

extern crate reqwest;
extern crate serde;
use std::io::Read;

#[derive(Debug)]
pub enum NodeType {
    Tezedge,
    Ocaml,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn test_heads() {
    let rust_head = match get_head(NodeType::Tezedge) {
        Ok(v) => v,
        Err(e) => panic!("Invalid json: {}", e),
    };

    let block_id: String = rust_head["hash"].to_string();

    let ocaml_block = match get_block(&block_id) {
        Ok(v) => v,
        Err(e) => panic!("Invalid json: {}", e),
    };

    // TODO: make it more obvious in the output
    //              actual      expected
    assert_json_eq!(rust_head, ocaml_block);
}

fn get_block(block_id: &String) -> Result<serde_json::value::Value> {
    let url = format!(
        "{}{}",
        "http://46.101.160.245:3000/chains/main/blocks/",
        block_id.replace("\"", "")
    );

    let mut res = reqwest::blocking::get(&url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(serde_json::from_str(&body)?)
}

fn get_head(node_type: NodeType) -> Result<serde_json::value::Value> {
    let url = match node_type {
        NodeType::Ocaml => "http://46.101.160.245:3000/chains/main/blocks/head", // reference Ocaml node
        NodeType::Tezedge => "http://node-run:18732/chains/main/blocks/head", // locally built Tezedge node
    };

    let mut res = reqwest::blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(serde_json::from_str(&body)?)
}

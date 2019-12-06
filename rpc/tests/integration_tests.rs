// #[macro_use]
// extern crate assert_json_diff;
extern crate reqwest;
extern crate serde;

use serde::Deserialize;

#[derive(Debug)]
pub enum NodeType {
    Tezedge,
    Ocaml,
}

#[derive(Debug, Deserialize)]
struct Bootstrapped {
    block: String,
    timestamp: String,
}

use chrono::{DateTime, Utc};
use std::thread;
use std::time::Duration;

#[test]
fn test_heads() {
    wait_to_bootsrapp();
    println!("Good!")

    // let rust_head = match get_head(NodeType::Tezedge) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Invalid json: {}", e),
    // };

    // let block_id: String = rust_head["hash"].to_string();

    // let ocaml_block = match get_block(&block_id) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Invalid json: {}", e),
    // };

    // // TODO: make it more obvious in the output
    // //              actual      expected
    // assert_json_eq!(rust_head, ocaml_block);
}

// #[test]
// fn test_first_1k_heads() {
//     // should we use recursion?
//     // TODO: test recursion

//     let next_block = "BM9xFVaVv6mi7ckPbTgxEe7TStcfFmteJCpafUZcn75qi2wAHrC"; // 1000th

// }

fn wait_to_bootsrapp() {
    let connect_thread = thread::spawn(|| loop {
        let resp = reqwest::blocking::get("http://tezedge-node-run:18732/monitor/bootstrapped");
        if resp.unwrap().status().is_success() {
            break;
        } else {
            thread::sleep(Duration::from_secs(10));
        }
    });

    let bootstrap_monitoring_thread = thread::spawn(|| loop {
        match is_bootstrapped() {
            Ok(s) => {
                let desired_timestamp =
                    DateTime::parse_from_rfc3339("2019-09-28T08:14:24Z").unwrap();
                let block_timestamp = DateTime::parse_from_rfc3339(&s).unwrap();

                if block_timestamp >= desired_timestamp {
                    println!("Done Bootstrapping");
                    break;
                } else {
                    println!("Bootstrapping . . . timestamp: {}", s);
                    thread::sleep(Duration::from_secs(10));
                }
            }
            Err(e) => {
                println!("Error in bootstrap check: {}", e);
                thread::sleep(Duration::from_secs(10));
            }
        }
    });

    connect_thread.join();
    bootstrap_monitoring_thread.join();
}

#[allow(dead_code)]
fn is_bootstrapped() -> Result<String, reqwest::Error> {
    let response: String =
        reqwest::blocking::get("http://tezedge-node-run:18732/monitor/bootstrapped")?.text()?;

    // hack to handle case when the node did not start the bootstrapping process and retruns timestamp with int 0
    if response.contains(r#""timestamp":0"#) {
        Ok(String::new())
    } else {
        let response_node: Bootstrapped =
            serde_json::from_str(&response).expect("JSON was not well-formatted");

        // parse timestamp to int form request
        // let datetime_node = DateTime::parse_from_rfc3339(&response_node.timestamp.to_string()).unwrap();
        Ok(response_node.timestamp.to_string())
    }
}

fn get_block(block_id: &String) -> Result<serde_json::value::Value, serde_json::error::Error> {
    let url = format!(
        "{}{}",
        "http://ocaml-node-run:8732/chains/main/blocks/",
        block_id.replace("\"", "")
    );

    let res = match reqwest::blocking::get(&url) {
        Ok(v) => v,
        Err(e) => panic!("Request for getting block failed: {}", e),
    };
    //let mut body = String::new();
    //res.read_to_string(&mut body);

    serde_json::from_str(&res.text().unwrap())
}

fn get_head(node_type: NodeType) -> Result<serde_json::value::Value, serde_json::error::Error> {
    let url = match node_type {
        NodeType::Ocaml => "http://ocaml-node-run:8732/chains/main/blocks/head", // reference Ocaml node
        NodeType::Tezedge => "http://tezedge-node-run:18732/chains/main/blocks/head", // locally built Tezedge node
    };

    let res = match reqwest::blocking::get(url) {
        Ok(v) => v,
        Err(e) => panic!("Request for getting block failed: {}", e),
    };
    //let mut body = String::new();
    //res.read_to_string(&mut body);

    serde_json::from_str(&res.text().unwrap())
}

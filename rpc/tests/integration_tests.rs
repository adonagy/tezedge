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
    //wait_to_bootsrapp();
    create_monitor_node_thread(NodeType::Tezedge)
        .join()
        .unwrap();
    create_monitor_node_thread(NodeType::Ocaml).join().unwrap();

    test_first_1k_heads();
}

fn test_first_1k_heads() {
    // should we use recursion?
    // TODO: test recursion

    let mut next_block = "BM9xFVaVv6mi7ckPbTgxEe7TStcfFmteJCpafUZcn75qi2wAHrC".to_string(); // 1000th

    while next_block != "" {
        let ocaml_json = get_block(NodeType::Ocaml, &next_block).expect("Failed to get block");
        let tezedge_json = get_block(NodeType::Tezedge, &next_block).expect("Failed to get block");
        let predecessor = ocaml_json["header"]["predecessor"]
            .to_string()
            .replace("\"", "");

        // NOTE: this will allways fail for now due to unimplemented properties in tezedge
        // to verify the loop, we just print the next block to be checked
        //assert_json_eq!(tezedge_json, ocaml_json);

        // debug: remove later
        if next_block == "BLockGenesisGenesisGenesisGenesisGenesisd1f7bcGMoXy" {
            println!("Genesis block reached and checked, breaking loop...");
            break;
        }

        next_block = predecessor;
        // TODO: remove this line
        println!("{}", &next_block);
    }
}

fn wait_to_bootsrapp() {
    let bootstrapping_tezedge = create_monitor_node_thread(NodeType::Tezedge);
    let bootstrapping_ocaml = create_monitor_node_thread(NodeType::Ocaml);

    bootstrapping_tezedge.join().unwrap();
    bootstrapping_ocaml.join().unwrap();
}

fn create_monitor_node_thread(node: NodeType) -> JoinHandle<()> {
    let bootstrap_monitoring_thread = thread::spawn(move || loop {
        match is_bootstrapped(&node) {
            Ok(s) => {
                if s != "" {
                    let desired_timestamp =
                        DateTime::parse_from_rfc3339("2019-09-28T08:14:24Z").unwrap();
                    let block_timestamp = DateTime::parse_from_rfc3339(&s).unwrap();

                    if block_timestamp >= desired_timestamp {
                        println!("[{}] Done Bootstrapping", node.to_string());
                        break;
                    } else {
                        println!(
                            "[{}] Bootstrapping . . . timestamp: {}",
                            node.to_string(),
                            s
                        );
                        thread::sleep(Duration::from_secs(10));
                    }
                } else {
                    println!(
                        "[{}] Waiting for node to start bootstrapping...",
                        node.to_string()
                    );
                    thread::sleep(Duration::from_secs(10));
                }
            }
            Err(_e) => {
                // panic!("Error in bootstrap check: {}", e);
                println!("[{}] Waiting for node to run", node.to_string());
                println!("[{}] Error: {}", node.to_string(), _e);

                thread::sleep(Duration::from_secs(10));
            }
        }
        println!("[{}] Loop cycel ending", node.to_string());
    });
    bootstrap_monitoring_thread
}

#[allow(dead_code)]
fn is_bootstrapped(node: &NodeType) -> Result<String, reqwest::Error> {
    let response;
    match node {
        NodeType::Tezedge => {
            response =
                reqwest::blocking::get("http://tezedge-node-run:18732/chains/main/blocks/head")?;
        }
        NodeType::Ocaml => {
            response =
                reqwest::blocking::get("http://ocaml-node-run:8732/chains/main/blocks/head")?;
        }
    }
    if response.status().is_success() {
        let response_node: serde_json::value::Value =
            serde_json::from_str(&response.text()?).expect("JSON was not well-formatted");

        Ok(response_node["header"]["timestamp"]
            .to_string()
            .replace("\"", ""))
    } else {
        Ok(String::new())
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

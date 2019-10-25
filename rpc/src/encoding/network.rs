use serde::{Serialize, Deserialize};
use super::base_types::*;

pub type PublicKeyHash = UniString;

//  [ { "incoming": boolean,
//      "peer_id": $Crypto_box.Public_key_hash,
//      "id_point": $p2p_connection.id,
//      "remote_socket_port": integer ∈ [0, 2^16-1],
//      "announced_version": $network_version,
//      "private": boolean,
//      "local_metadata":
//        { "disable_mempool": boolean,
//          "private_node": boolean },
//      "remote_metadata":
//        { "disable_mempool": boolean,
//          "private_node": boolean } } ... ]

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInfo {
    incoming: bool,
    peer_id: PublicKeyHash,
    id_point: P2PConnectionId,
}

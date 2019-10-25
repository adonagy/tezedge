use serde::{Serialize, Deserialize};
use crate::monitors::PeerMonitor;

#[derive(Debug, Clone)]
pub enum GetPeerStats {
    Request,
    Response(P2PStats),
}

// { "total_sent": $int64,
//      "total_recv": $int64,
//      "current_inflow": integer ∈ [-2^30-2, 2^30+2],
//      "current_outflow": integer ∈ [-2^30-2, 2^30+2] }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct P2PStats {
    total_sent: i64,
    total_recv: i64,
    current_inflow: i32,
    current_outflow: i32,
}

impl P2PStats {
    pub fn new(total_sent: i64, total_recv: i64, current_inflow: i32, current_outflow: i32) -> Self {
        Self {
            total_sent,
            total_recv,
            current_inflow,
            current_outflow,
        }
    }

    pub fn incoming(total_recv: i64, current_inflow: i32) -> Self {
        Self::new(0, total_recv, current_inflow, 0)
    }
}

impl From<IntoIterator<Item=&PeerMonitor>> for P2PStats {
    fn from(vals: Vec<PeerMonitor>) -> Self {
        let (sent, recv, inflow, outflow) = vals.into_iter().fold((0, 0, 0, 0, ), |(sent, recv, inflow, outflow), monitor| {
            (sent + monitor.total_transferred as i64, recv, inflow + monitor.current_speed() as i32, outflow)
        });
        Self::new(sent, recv, inflow, outflow)
    }
}
// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT
#![forbid(unsafe_code)]

use std::hash::{Hash, Hasher};
use chrono::DateTime;
use tokio::time::Duration;
use crate::node::OcamlNode;
use std::collections::{VecDeque, HashSet};
use std::sync::{Arc, RwLock};
use std::convert::TryInto;
use chrono::Utc;

use serde::Serialize;
use slog::{info, crit, Logger};
use percentage::{Percentage, PercentageInteger};

use shell::stats::memory::ProcessMemoryStats;

use crate::display_info::DiskData;
use crate::node::{Node, TezedgeNode, OCAML_PORT, TEZEDGE_PORT};
use crate::monitors::TEZEDGE_VOLUME_PATH;

pub type ResourceUtilizationStorage = Arc<RwLock<VecDeque<ResourceUtilization>>>;

/// The max capacity of the VecDeque holding the measurements
pub const MEASUREMENTS_MAX_CAPACITY: usize = 1440;

// TODO: pass these as parameters

// maximum diskspace allowed (critical threshold)
const DISK_THRESHOLD: u64 = 107_374_182_400;

// maximum ram usage to allert (critical threshold)
const RAM_THRESHOLD: u64 = 10_737_418_240;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum AlertLevel {
    NonAlert = 1,
    Info = 2,
    Warning = 3,
    Severe = 4,
    Critical = 5,
}

impl AlertLevel {
    pub fn value(&self) -> PercentageInteger {
        match *self {
            AlertLevel::NonAlert => Percentage::from(0),
            AlertLevel::Info => Percentage::from(40),
            AlertLevel::Warning => Percentage::from(60),
            AlertLevel::Severe => Percentage::from(80),
            AlertLevel::Critical => Percentage::from(100),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AlertKind {
    Disk,
    Memory,
    Cpu,
    NodeStucked,
}

#[derive(Clone, Copy, Debug)]
pub struct AllertThresholds {
    ram: u64,
    disk: u64,
    head: i64,
}

impl AllertThresholds {
    pub fn new(ram: u64, disk: u64, head: i64) -> Self {
        Self {
            ram,
            disk, head, 
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResourceMonitor {
    ocaml_resource_utilization: ResourceUtilizationStorage,
    tezedge_resource_utilization: ResourceUtilizationStorage,
    latest_head_timestamp: Option<i64>,
    alerts: HashSet<MonitorAlert>,
    log: Logger,
}

#[derive(Clone, Debug, Eq)]
pub struct MonitorAlert {
    level: AlertLevel,
    kind: AlertKind,
}

// implement PartialEq and Hash traits manually to achieve that each kind of alert is only once in the HashSet
impl PartialEq for MonitorAlert {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Hash for MonitorAlert {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl MonitorAlert {
    pub fn new(level: AlertLevel, kind: AlertKind) -> Self {
        Self {
            level,
            kind,
        }
    }

    pub fn assign_alert(kind: AlertKind, threshold: u64, value: u64) -> Self {
        println!("kind: {:?}", kind);
        println!("threshold: {:?}", threshold);
        println!("value: {:?}", value);
        let level = if value >= AlertLevel::Critical.value().apply_to(threshold) {
            AlertLevel::Critical
        } else if value >= AlertLevel::Severe.value().apply_to(threshold) {
            AlertLevel::Severe
        } else if value >= AlertLevel::Warning.value().apply_to(threshold) {
            AlertLevel::Warning
        } else if value >= AlertLevel::Info.value().apply_to(threshold){
            AlertLevel::Info
        } else {
            AlertLevel::NonAlert
        };

        Self {
            level,
            kind
        }
    }

    // pub fn changed() -> Result<(), failure::Error> {

    //     Ok(())
    // }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MemoryStats {
    node: ProcessMemoryStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_runners: Option<ProcessMemoryStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validators: Option<ProcessMemoryStats>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResourceUtilization {
    timestamp: i64,
    memory: MemoryStats,
    disk: DiskData,
    cpu: CpuStats,
}

#[derive(Clone, Debug, Serialize)]
pub struct CpuStats {
    node: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_runners: Option<i32>,
}

impl ResourceMonitor {
    pub fn new(
        ocaml_resource_utilization: ResourceUtilizationStorage,
        tezedge_resource_utilization: ResourceUtilizationStorage,
        latest_head_timestamp: Option<i64>,
        alerts: HashSet<MonitorAlert>,
        log: Logger,
    ) -> Self {
        Self {
            ocaml_resource_utilization,
            tezedge_resource_utilization,
            latest_head_timestamp,
            alerts,
            log,
        }
    }

    pub async fn take_measurement(&mut self) -> Result<(), failure::Error> {
        // memory rpc
        let tezedge_node = TezedgeNode::collect_memory_data(&self.log, TEZEDGE_PORT).await?;
        let ocaml_node = OcamlNode::collect_memory_data(&self.log, OCAML_PORT).await?;

        // protocol runner memory rpc
        let protocol_runners =
            TezedgeNode::collect_protocol_runners_memory_stats(TEZEDGE_PORT).await?;

        // tezos validators memory data
        let tezos_validators = OcamlNode::collect_validator_memory_stats()?;

        // collect disk stats
        let tezedge_disk = TezedgeNode::collect_disk_data()?;
        let ocaml_disk = OcamlNode::collect_disk_data()?;

        // cpu stats
        let tezedge_cpu = TezedgeNode::collect_cpu_data("light-node")?;
        let protocol_runners_cpu = TezedgeNode::collect_cpu_data("protocol-runner")?;
        let ocaml_cpu = OcamlNode::collect_cpu_data("tezos-node")?;

        let tezedge_resources = ResourceUtilization {
            timestamp: chrono::Local::now().timestamp(),
            memory: MemoryStats {
                node: tezedge_node,
                protocol_runners: Some(protocol_runners),
                validators: None,
            },
            disk: tezedge_disk,
            cpu: CpuStats {
                node: tezedge_cpu,
                protocol_runners: Some(protocol_runners_cpu),
            },
        };

        let ocaml_resources = ResourceUtilization {
            timestamp: chrono::Local::now().timestamp(),
            memory: MemoryStats {
                node: ocaml_node,
                protocol_runners: None,
                validators: Some(tezos_validators),
            },
            disk: ocaml_disk,
            cpu: CpuStats {
                node: ocaml_cpu,
                protocol_runners: None,
            },
        };

        // custom block to drop the write lock as soon as possible
        {
            let ocaml_resources_ref = &mut *self.ocaml_resource_utilization.write().unwrap();
            let tezedge_resources_ref = &mut *self.tezedge_resource_utilization.write().unwrap();

            // if we are about to exceed the max capacity, remove the last element in the VecDeque
            if ocaml_resources_ref.len() == MEASUREMENTS_MAX_CAPACITY
                && tezedge_resources_ref.len() == MEASUREMENTS_MAX_CAPACITY
            {
                ocaml_resources_ref.pop_back();
                tezedge_resources_ref.pop_back();
            }

            tezedge_resources_ref.push_front(tezedge_resources.clone());
            ocaml_resources_ref.push_front(ocaml_resources);
        }

        // handle alerts
        {
            // TODO: pass this as an arg from main
            let thresholds = AllertThresholds::new(RAM_THRESHOLD, DISK_THRESHOLD, 300);

            //let ram_percent = Percentage::from(RAM_THRESHOLD);
            let ram_total = tezedge_resources.memory.node.resident_mem() + tezedge_resources.memory.protocol_runners.unwrap_or(ProcessMemoryStats::default()).resident_mem();
            let ram_alert = MonitorAlert::assign_alert(AlertKind::Memory, thresholds.ram, ram_total.try_into().unwrap_or(0));

            // gets the total space on the filesystem of the specified path
            let free_disk_space = fs2::free_space(TEZEDGE_VOLUME_PATH)?;
            // let total_disk_space = fs2::total_space(TEZEDGE_VOLUME_PATH)?;
            let total_disk_space = fs2::total_space(TEZEDGE_VOLUME_PATH)?;
            let disk_alert = MonitorAlert::assign_alert(AlertKind::Disk, thresholds.disk, total_disk_space - free_disk_space);

            let head = TezedgeNode::collect_head_data(&self.log, TEZEDGE_PORT).await?;
            let block_timestamp = head.timestamp();
            let time = Utc::now().timestamp();

            let block_time = DateTime::parse_from_rfc3339(block_timestamp)
                .map(|dt| dt.timestamp())
                .ok();

            if let Some(timestamp) = self.latest_head_timestamp {
                // alert if the head is not genesis, it has not changed and the last timestamp extends the defined threshold
                if Some(timestamp) == block_time && (time - timestamp) > thresholds.head {
                    let head_alert = MonitorAlert::new(AlertLevel::Critical, AlertKind::NodeStucked);
                    crit!(self.log, "HEAD ALERT: {:?}", head_alert);
                    // make this allert only critical and NonAlert
                    if self.alerts.contains(&head_alert) {
                        self.alerts.remove(&head_alert);
                    } else {
                        self.alerts.insert(head_alert);
                        // send alert to slack
                    }
                }
            }

            crit!(self.log, "RAM ALERT: {:?}", &ram_alert);
            crit!(self.log, "DISK ALERT: {:?}", &disk_alert);

            if self.alerts.contains(&disk_alert) {
                if let Some(previous_alert) = self.alerts.get(&disk_alert) {
                    if disk_alert.level > previous_alert.level {
                        info!(self.log, "More severe alert detected!");
                        self.alerts.replace(disk_alert);
                    }
                }
            }

            if self.alerts.contains(&ram_alert) {
                if let Some(previous_alert) = self.alerts.get(&ram_alert) {
                    if ram_alert.level > previous_alert.level {
                        self.alerts.replace(ram_alert);
                    }
                }
            }

            self.latest_head_timestamp = block_time;
        }

        Ok(())
    }
}

fn handle_alerts() {
    
}
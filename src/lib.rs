use anyhow::Result;
use pnet::datalink::NetworkInterface;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};
use uuid::Uuid;

pub mod data;
mod geoip;
mod packet;
mod stats;
pub mod tasks;
mod traceroute;

pub use crate::{
    data::DbHandle,
    traceroute::{Config, TraceRoute},
};

/// Result of a single query execution. A query can either succeed and return
/// the round-trip time and address of a hop, or yield a timeout or fail.
#[derive(Debug, Clone)]
pub enum TraceQuery {
    Success {
        /// Round-Trip Time
        rtt: Duration,
        /// IP address of a remote node
        addr: IpAddr,
    },
    Timeout,
    Failure(String),
}

/// Single traceroute hop containing TTL, the source and destination IP of a
/// trace, and a vector of traceroute query results
#[derive(Debug, Clone)]
pub struct Hop {
    /// The unique trace id that this hop was part of.
    pub trace: Uuid,
    /// Current Time-To-Live.
    pub ttl: u8,
    /// The source of the trace.
    pub source: Ipv4Addr,
    /// The destination of the trace.
    pub destination: Ipv4Addr,
    /// Traceroute query results.
    pub queries: Vec<TraceQuery>,
}

impl Hop {
    fn is_ip(&self, ip: Ipv4Addr) -> bool {
        self.queries.iter().any(|query| match query {
            TraceQuery::Success { addr, .. } => addr == &IpAddr::V4(ip),
            _ => false,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub id: Uuid,
    pub route: Route,
}

impl Trace {
    /// Creates new instance of a trace.
    pub fn new(source: Ipv4Addr, destination: Ipv4Addr) -> Self {
        let id = Uuid::new_v4();
        let route = Route {
            source,
            destination,
        };

        Trace { id, route }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
}

/// Fetch the IP address for an network interface. If no interface is provided
/// (`None`) return the IP address of the default network interface. Otherwise
/// return the IP address of the provided network interface.
pub fn interface_ip(interface: Option<NetworkInterface>) -> Result<Ipv4Addr> {
    match interface {
        Some(interface) => traceroute::interface_ip(interface),
        None => traceroute::interface_ip(traceroute::default_interface()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportHop {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub trace: Uuid,
    pub ttl: u8,
    pub query: u8,
    pub query_result: String,
    pub addr: Option<Ipv4Addr>,
    // FIXME: the rtt is a i128
    pub rtt: Option<String>,
    pub hop_mean_ms: Option<String>,
    pub hop_median_ms: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub country_code_iso3: Option<String>,
    pub country_capital: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub timezone: Option<String>,
    pub utc_offset: Option<String>,
    pub asn: Option<String>,
    pub org: Option<String>,
}

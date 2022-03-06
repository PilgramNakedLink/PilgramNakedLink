use anyhow::Result;
use pnet::{
    datalink::{self, MacAddr, NetworkInterface},
    packet::{icmp::IcmpTypes, ip::IpNextHeaderProtocols, Packet},
    transport::{
        TransportProtocol::Ipv4,
        TransportSender, {icmp_packet_iter, transport_channel, TransportChannelType},
    },
};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    time::{Duration, SystemTime},
};

use crate::{packet::PacketBuilder, Hop, Trace, TraceQuery};

/// List all available interfaces on this machine.
pub fn available_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = datalink::interfaces();

    let available_interfaces: Vec<NetworkInterface>;

    available_interfaces = if cfg!(target_family = "windows") {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
                    && e.ips.iter().any(|ip| ip.ip().to_string() != "0.0.0.0")
            })
            .collect()
    } else {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.is_up()
                    && !e.is_loopback()
                    && e.ips.iter().any(|ip| ip.is_ipv4())
                    && e.mac.is_some()
                // && e.mac.unwrap() != MacAddr::zero()
            })
            .collect()
    };

    available_interfaces
}

/// Return the default network interface.
pub fn default_interface() -> NetworkInterface {
    let available_interfaces = available_interfaces();

    available_interfaces
        .get(0)
        .expect("no interfaces available")
        .clone()
}

/// Extract the IP address of the network interface.
pub fn interface_ip(interface: NetworkInterface) -> Result<Ipv4Addr> {
    let ip = interface
        .ips
        .iter()
        .find(|i| i.is_ipv4())
        .expect("couldn't get interface IP")
        .ip()
        .to_string();

    Ok(Ipv4Addr::from_str(ip.as_str())?)
}

/// Traceroute configurations
#[derive(Debug)]
pub struct Config {
    port: u16,
    max_hops: u8,
    tries: u8,
    timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 33434,
            max_hops: 30,
            tries: 3,
            timeout: Duration::from_secs(5),
        }
    }
}

impl Config {
    /// Builder: Port for traceroute. Will be incremented on every query (except for TCP-based traceroute)
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Builder: Maximum number of hops
    pub fn with_max_hops(mut self, max_hops: u8) -> Self {
        self.max_hops = max_hops;
        self
    }

    /// Builder: number of queries for each hop.
    pub fn with_tries(mut self, tries: u8) -> Self {
        self.tries = tries;
        self
    }

    /// Builder: Timeout per query
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Duration::from_millis(timeout);
        self
    }
}

/// Traceroute instance containing destination address and configurations. A
/// `TraceRoute` executes an actual trace route and produces hops for a trace.
pub struct TraceRoute {
    pub trace: Trace,
    config: Config,
    packet_builder: PacketBuilder,
    tx: TransportSender,
}

impl TraceRoute {
    /// Creates new instance of Traceroute
    pub fn new(source: Ipv4Addr, destination: Ipv4Addr, config: Config) -> Self {
        let trace = Trace::new(source, destination);
        let packet_builder = PacketBuilder::new(source, destination);
        let transport = transport_channel(
            4096,
            TransportChannelType::Layer3(IpNextHeaderProtocols::Udp),
        );

        let (tx, _) = match transport {
            Ok((tx, rx)) => (tx, rx),
            Err(e) => panic!("layer3: unable to create channel: {}", e),
        };

        TraceRoute {
            trace,
            config,
            packet_builder,
            tx,
        }
    }

    /// Return an iterator of hops over this trace.
    pub fn iter(&mut self) -> TraceRouteIter<'_> {
        TraceRouteIter {
            done: false,
            ttl: 1,
            traceroute: self,
        }
    }

    /// Run a complete traceroute and return a list of all hops that are part of
    /// this trace route.
    pub fn traceroute(&mut self) -> Vec<Hop> {
        let mut hops: Vec<Hop> = vec![];

        for hop in self.iter() {
            hops.push(hop);
        }

        hops
    }

    /// Yield the next hop of this trace.
    fn hop(&mut self, ttl: u8) -> Hop {
        let mut queries: Vec<TraceQuery> = vec![];

        for _ in 0..self.config.tries {
            let packet = self.packet_builder.build_packet(ttl, self.config.port);
            let query_result = self.query(packet);
            queries.push(query_result);
        }

        Hop {
            queries,
            ttl,
            trace: self.trace.id,
            source: self.trace.route.source,
            destination: self.trace.route.destination,
        }
    }

    /// Runs a query to the destination and returns RTT and IP of the router where
    /// time-to-live-exceeded. Doesn't increase TTL
    fn query(&mut self, packet: impl Packet) -> TraceQuery {
        let now = SystemTime::now();
        let protocol = TransportChannelType::Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

        let (_, mut receiver) = match transport_channel(4096, protocol) {
            Ok((tx, rx)) => (tx, rx),
            Err(e) => panic!("layer4: unable to create channel: {}", e),
        };

        let mut iter = icmp_packet_iter(&mut receiver);

        match self
            .tx
            .send_to(packet, IpAddr::V4(self.trace.route.destination))
        {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Could not send packet, make sure this program has needed privilages, Error<{}>",
                    e.to_string()
                );
            }
        }

        let next = iter.next_with_timeout(self.config.timeout);

        match next {
            Ok(Some((header, addr))) => match header.get_icmp_type() {
                IcmpTypes::TimeExceeded
                | IcmpTypes::EchoReply
                | IcmpTypes::DestinationUnreachable => TraceQuery::Success {
                    rtt: now.elapsed().unwrap_or_else(|_| Duration::from_millis(0)),
                    addr,
                },
                _ => TraceQuery::Failure("wrong packet".to_string()),
            },
            Ok(None) => TraceQuery::Timeout,
            Err(e) => TraceQuery::Failure(e.to_string()),
        }
    }
}

/// An iterator over a trace. Returns individual hops of a trace as it's
/// elements.
pub struct TraceRouteIter<'a> {
    ttl: u8,
    done: bool,
    traceroute: &'a mut TraceRoute,
}

impl<'a> TraceRouteIter<'a> {
    /// Increment the current TTL of this trace.
    fn increment_ttl(&mut self) {
        self.ttl += 1;
    }

    /// Test whether the trace has been completed or can be continued.
    fn is_finished(&self) -> bool {
        self.done || self.ttl >= self.traceroute.config.max_hops
    }
}

impl<'a> Iterator for TraceRouteIter<'a> {
    type Item = Hop;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished() {
            return None;
        }

        let hop = self.traceroute.hop(self.ttl);

        self.increment_ttl();

        if hop.is_ip(self.traceroute.trace.route.destination) {
            self.done = true;
        };

        Some(hop)
    }
}

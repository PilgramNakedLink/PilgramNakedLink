use pnet::packet::{ip::IpNextHeaderProtocols, ipv4, udp, Packet};
use rand::random;
use std::net::Ipv4Addr;

pub struct PacketBuilder {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
}

impl<'a> PacketBuilder {
    pub fn new(source_ip: Ipv4Addr, destination_ip: Ipv4Addr) -> Self {
        PacketBuilder {
            source_ip,
            destination_ip,
        }
    }

    pub fn build_packet(&self, ttl: u8, port: u16) -> impl Packet {
        Self::build_udp_packet(self.source_ip, self.destination_ip, ttl, port)
    }

    /// Create a new UDP packet
    fn build_udp_packet(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        ttl: u8,
        port: u16,
    ) -> ipv4::Ipv4Packet<'a> {
        const UDP_SIZE: usize = 64;
        const IPV4_SIZE: usize = ipv4::MutableIpv4Packet::minimum_packet_size() + UDP_SIZE;

        let mut udp_buf: Vec<u8> = vec![0; UDP_SIZE];
        let ipv4_buf: Vec<u8> = vec![0; IPV4_SIZE];

        let mut udp_packet = udp::MutableUdpPacket::new(&mut udp_buf[..]).unwrap();
        udp_packet.set_source(random::<u16>());
        udp_packet.set_destination(port);
        udp_packet.set_length(UDP_SIZE as u16);
        udp_packet.set_payload(&[0; UDP_SIZE - 8]);
        let csum = udp::ipv4_checksum(&udp_packet.to_immutable(), &source_ip, &destination_ip);
        udp_packet.set_checksum(csum);

        let mut ipv4_packet = ipv4::MutableIpv4Packet::owned(ipv4_buf).unwrap();
        ipv4_packet.set_header_length(5);
        ipv4_packet.set_fragment_offset(16384);
        ipv4_packet.set_identification(rand::random::<u16>());
        ipv4_packet.set_version(4);
        ipv4_packet.set_ttl(ttl);
        ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
        ipv4_packet.set_source(source_ip);
        ipv4_packet.set_destination(destination_ip);
        ipv4_packet.set_total_length(IPV4_SIZE as u16);
        ipv4_packet.set_payload(&udp_buf);

        let csum = ipv4::checksum(&ipv4_packet.to_immutable());
        ipv4_packet.set_checksum(csum);

        ipv4_packet.consume_to_immutable()
    }
}

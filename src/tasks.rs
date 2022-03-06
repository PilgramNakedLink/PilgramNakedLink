use anyhow::Result;
use std::{net::IpAddr, time::Duration};

use crate::{
    data::DbHandle,
    geoip, stats, {Hop, TraceQuery},
};

pub enum Task {
    HopLog(Hop),
    HopStats(Hop),
    HopGeoIp(Hop),
}

pub fn hop_log(hop: Hop) -> Result<()> {
    let queries = hop
        .queries
        .iter()
        .map(|q| match q {
            TraceQuery::Success { addr, rtt } => {
                format!("{} ({}ms)", addr, rtt.as_millis().to_string())
            }
            TraceQuery::Timeout => format!("*"),
            TraceQuery::Failure(_) => format!("X"),
        })
        .collect::<Vec<String>>()
        .join("  ");

    let msg = format!("{}: {}", hop.ttl, queries);

    println!("{}", msg);

    Ok(())
}

pub fn hop_stats(db: &DbHandle, hop: Hop) -> Result<()> {
    let mut durations = hop
        .queries
        .iter()
        .filter_map(|q| match q {
            &TraceQuery::Success { rtt, .. } => Some(rtt),
            _ => None,
        })
        .collect::<Vec<Duration>>();
    durations.sort_unstable();

    let stats = stats::HopStats::from_durations(&durations);

    db.insert_stats(hop, stats);

    Ok(())
}

pub fn hop_geoip(db: &DbHandle, hop: Hop) -> Result<()> {
    let mut idx = 1;
    for query in &hop.queries {
        match query {
            TraceQuery::Success { addr, .. } => match addr {
                IpAddr::V4(ipv4) => {
                    if !ipv4.is_private() {
                        let lookup_data = db
                            .show_geoip(&ipv4)
                            .or_else(|| geoip::fetch_ip_api(&addr).ok());

                        if let Some(ip_api_resp) = lookup_data {
                            db.insert_geoip(hop.clone(), idx, ip_api_resp);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
        idx += 1;
    }

    Ok(())
}

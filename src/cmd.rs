use anyhow::{Context, Error, Result};
use crossbeam_channel::bounded;
use std::{net::IpAddr, sync::Arc};

use tracer::{
    data::{migrate_db, DbHandle},
    interface_ip,
    tasks::{self, Task},
    Route, {Config, TraceRoute},
};

use crate::AppConfig;

pub(crate) fn init(cfg: AppConfig) -> Result<()> {
    migrate_db(&cfg.db)?;

    Ok(())
}

pub(crate) fn trace(cfg: AppConfig) -> Result<()> {
    let destination = cfg
        .destination
        .ok_or_else(|| Error::msg("destination address is missing"))?;
    let (snd1, rcv1) = bounded(1);
    let (snd2, _rcv2) = bounded::<tasks::Task>(1);
    let cpus = num_cpus::get();
    let n_workers = if cpus > 2 { cpus / 2 } else { 1 };

    let db = Arc::new(DbHandle::new(cfg.db).context("Failed to start database actor.")?);

    let source_ip = interface_ip(None)?;
    let destination_ip = match destination {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(ip) => ip.to_ipv4().unwrap(),
    };
    let config = Config::default();
    let mut traceroute = TraceRoute::new(source_ip, destination_ip, config);

    db.insert_route(traceroute.trace.route.clone());
    db.insert_trace(traceroute.trace.clone());

    crossbeam::scope(|s| {
        // The producer does the traces and pushes routes, traces and hops into
        // consumers for further processing.
        s.spawn(|_| {
            for hop in traceroute.iter() {
                db.insert_hop(hop.clone());

                snd1.send(Task::HopLog(hop.clone())).unwrap();
                snd1.send(Task::HopStats(hop.clone())).unwrap();
                snd1.send(Task::HopGeoIp(hop)).unwrap();
            }

            // Close the channel - this is necessary to exit
            // the for-loop in the worker
            drop(snd1);
        });

        // Each worker listens to incoming tasks and runs them as they come in.
        for _ in 0..n_workers {
            let (_sendr, recvr) = (snd2.clone(), rcv1.clone());
            let local_db = Arc::clone(&db);

            s.spawn(move |_| {
                for task in recvr.iter() {
                    match task {
                        Task::HopLog(hop) => tasks::hop_log(hop).unwrap(),
                        Task::HopStats(hop) => tasks::hop_stats(&local_db, hop).unwrap(),
                        Task::HopGeoIp(hop) => tasks::hop_geoip(&local_db, hop).unwrap(),
                    };
                }
            });
        }

        // Close the channel, otherwise sink will never
        // exit the for-loop
        drop(snd2);
    })
    .unwrap();

    db.shutdown();

    Ok(())
}

pub(crate) fn export(cfg: AppConfig) -> Result<()> {
    let destination = cfg
        .destination
        .ok_or_else(|| Error::msg("destination address is missing"))?;
    let db = Arc::new(DbHandle::new(cfg.db).context("Failed to start database actor.")?);

    let source_ip = interface_ip(None)?;
    let destination_ip = match destination {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(ip) => ip.to_ipv4().unwrap(),
    };

    let route = Route {
        source: source_ip,
        destination: destination_ip,
    };

    let hops = db.export_route(route);

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for hop in hops {
        wtr.serialize(hop)?;
    }
    wtr.flush()?;

    db.shutdown();

    Ok(())
}

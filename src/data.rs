use anyhow::Result;
use rusqlite::{params, types::Null};
use serde_rusqlite::{columns_from_statement, from_row_with_columns};
use std::{
    fmt::Debug,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::{mpsc, RwLock},
    thread,
};
use uuid::Uuid;

use crate::{geoip::IpApiResp, stats::HopStats, ExportHop, Hop, Route, Trace, TraceQuery};

pub fn migrate_db<P: AsRef<Path>>(path: P) -> Result<()> {
    let schema = include_str!("../ressources/schema.sql");
    let connection = rusqlite::Connection::open(path)?;
    connection.execute_batch(schema)?;
    Ok(())
}

#[derive(Debug)]
pub struct Manager {
    connection: rusqlite::Connection,
}

impl Manager {
    pub fn new(path: PathBuf) -> Result<Self> {
        let connection = Self::file(&path)?;
        connection.pragma_update(None, "foreign_keys", &1)?;

        Ok(Self { connection })
    }

    fn file<P: AsRef<Path>>(path: P) -> Result<rusqlite::Connection, rusqlite::Error> {
        rusqlite::Connection::open(path)
    }
}

struct Db {
    /// Messages to this actor are received on that channel.
    receiver: mpsc::Receiver<DbMessage>,
    /// Obtain a write lock for messages that require write access to the
    /// database.
    write_lock: RwLock<()>,
    /// The store interacts with persisted data.
    store: Store,
}

enum DbMessage {
    InsertRoute {
        route: Route,
        respond_to: mpsc::SyncSender<()>,
    },

    InsertTrace {
        trace: Trace,
        respond_to: mpsc::SyncSender<()>,
    },

    InsertHop {
        hop: Hop,
        respond_to: mpsc::SyncSender<()>,
    },

    InsertStats {
        hop: Hop,
        stats: HopStats,
        respond_to: mpsc::SyncSender<()>,
    },

    InsertGeoip {
        hop: Hop,
        query: u8,
        geoip: IpApiResp,
        respond_to: mpsc::SyncSender<()>,
    },

    ExportHop {
        route: Route,
        respond_to: mpsc::SyncSender<Vec<ExportHop>>,
    },

    ShowGeoip {
        addr: Ipv4Addr,
        respond_to: mpsc::SyncSender<Option<IpApiResp>>,
    },

    Shutdown,
}

impl Db {
    fn new(db: Manager, receiver: mpsc::Receiver<DbMessage>) -> Result<Db> {
        Ok(Db {
            receiver,
            write_lock: RwLock::new(()),
            store: Store { db },
        })
    }

    fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            if let DbMessage::Shutdown = msg {
                break;
            }
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, msg: DbMessage) {
        match msg {
            DbMessage::InsertRoute { route, respond_to } => {
                let _ = self.write_lock.write().unwrap();
                self.store
                    .insert_route(&route.source, &route.destination)
                    .expect("inserting a route");

                let _ = respond_to.send(());
            }

            DbMessage::InsertTrace { trace, respond_to } => {
                let _ = self.write_lock.write().unwrap();
                self.store
                    .insert_trace(&trace.route.source, &trace.route.destination, &trace.id)
                    .expect("inserting a trace");

                let _ = respond_to.send(());
            }

            DbMessage::InsertHop { hop, respond_to } => {
                let _ = self.write_lock.write().unwrap();
                self.store
                    .insert_hop(&hop.trace, hop.ttl, hop.queries)
                    .expect("inserting a hop");

                let _ = respond_to.send(());
            }

            DbMessage::InsertStats {
                hop,
                stats,
                respond_to,
            } => {
                let _ = self.write_lock.write().unwrap();
                self.store
                    .insert_stats(&hop.trace, hop.ttl, &stats)
                    .expect("inserting hop stats");

                let _ = respond_to.send(());
            }

            DbMessage::InsertGeoip {
                hop,
                query,
                geoip,
                respond_to,
            } => {
                let _ = self.write_lock.write().unwrap();
                self.store
                    .insert_geoip(&hop.trace, hop.ttl, query, &geoip)
                    .expect("inserting hop geoip");

                let _ = respond_to.send(());
            }

            DbMessage::ExportHop { route, respond_to } => {
                let hops = self
                    .store
                    .export_route(&route.source, &route.destination)
                    .expect("exporting a route");

                let _ = respond_to.send(hops);
            }

            DbMessage::ShowGeoip { addr, respond_to } => {
                let data = match self.store.show_geoip_for_addr(&addr) {
                    Ok(data) => Some(data),
                    Err(_) => None,
                };

                let _ = respond_to.send(data);
            }

            // The shutdown message is handled in the run method.
            DbMessage::Shutdown => {
                unreachable!();
            }
        }
    }
}

#[derive(Clone)]
pub struct DbHandle {
    sender: mpsc::SyncSender<DbMessage>,
}

impl DbHandle {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(100);
        let manager = Manager::new(db_path)?;
        let mut actor = Db::new(manager, receiver)?;
        let _handle = thread::spawn(move || actor.run());

        Ok(Self { sender })
    }

    pub fn insert_route(&self, route: Route) {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::InsertRoute {
            route,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn insert_trace(&self, trace: Trace) {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::InsertTrace {
            trace,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn insert_hop(&self, hop: Hop) {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::InsertHop {
            hop,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn insert_stats(&self, hop: Hop, stats: HopStats) {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::InsertStats {
            hop,
            stats,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn insert_geoip(&self, hop: Hop, query: u8, geoip: IpApiResp) {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::InsertGeoip {
            hop,
            query,
            geoip,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn export_route(&self, route: Route) -> Vec<ExportHop> {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::ExportHop {
            route,
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn show_geoip(&self, addr: &Ipv4Addr) -> Option<IpApiResp> {
        let (send, recv) = mpsc::sync_channel(1);

        let msg = DbMessage::ShowGeoip {
            addr: addr.clone(),
            respond_to: send,
        };

        let _ = self.sender.send(msg);
        recv.recv().expect("Db has been killed")
    }

    pub fn shutdown(&self) {
        // FIXME: handle shutdown gracefully and wait for the queue to be empty
        let _ = self.sender.send(DbMessage::Shutdown);
    }
}

struct Store {
    db: Manager,
}

impl Store {
    fn show_route_id(&self, source: &Ipv4Addr, destination: &Ipv4Addr) -> Result<i64> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/show-route.sql"))?;

        let route_id: i64 = stmt.query_row(
            params![&source.to_string(), &destination.to_string()],
            |row| row.get(0),
        )?;

        Ok(route_id)
    }

    fn show_trace_id(&self, trace: &Uuid) -> Result<i64> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/show-trace.sql"))?;

        let trace_id: i64 = stmt.query_row(params![trace.to_string()], |row| row.get(0))?;

        Ok(trace_id)
    }

    fn show_hop_ids(&self, trace: &Uuid, ttl: u8) -> Result<Vec<i64>> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/show-hops.sql"))?;

        let trace_id = self.show_trace_id(&trace)?;
        let rows = stmt.query_map(params![ttl, trace_id], |row| row.get(0))?;

        let mut hop_ids: Vec<i64> = Vec::new();

        for row in rows {
            hop_ids.push(row?);
        }

        Ok(hop_ids)
    }

    fn show_hop_id(&self, trace: &Uuid, ttl: u8, query: u8) -> Result<i64> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/show-hop.sql"))?;

        let trace_id = self.show_trace_id(&trace)?;
        let hop_id: i64 = stmt.query_row(params![ttl, query, trace_id], |row| row.get(0))?;

        Ok(hop_id)
    }

    fn insert_route(&self, source: &Ipv4Addr, destination: &Ipv4Addr) -> Result<i64> {
        let conn = &self.db.connection;

        let mut stmt = conn.prepare_cached(include_str!("sql/insert-route.sql"))?;

        stmt.execute(params![&source.to_string(), &destination.to_string()])?;
        let route_id = self.show_route_id(&source, &destination)?;

        Ok(route_id)
    }

    fn insert_trace(&self, source: &Ipv4Addr, destination: &Ipv4Addr, trace: &Uuid) -> Result<i64> {
        let conn = &self.db.connection;

        let mut stmt = conn.prepare_cached(include_str!("sql/insert-trace.sql"))?;

        let route_id = self.show_route_id(&source, &destination)?;
        stmt.execute(params![trace.to_string(), route_id])?;
        let trace_id = self.show_trace_id(&trace)?;

        Ok(trace_id)
    }

    fn insert_hop(&self, trace: &Uuid, ttl: u8, queries: Vec<TraceQuery>) -> Result<Vec<i64>> {
        let conn = &self.db.connection;

        let mut stmt = conn.prepare_cached(include_str!("sql/insert-hop.sql"))?;

        let trace_id = self.show_trace_id(&trace)?;
        let mut idx = 1;
        for query in queries {
            match query {
                TraceQuery::Success { addr, rtt } => {
                    stmt.execute(params![
                        ttl,
                        trace_id,
                        idx,
                        "success",
                        &addr.to_string(),
                        &rtt.as_millis().to_string()
                    ])?;
                }
                TraceQuery::Timeout => {
                    stmt.execute(params![ttl, trace_id, idx, "timeout", Null, Null])?;
                }
                TraceQuery::Failure(_) => {
                    stmt.execute(params![ttl, trace_id, idx, "fail", Null, Null])?;
                }
            };
            idx += 1;
        }
        let hop_ids = self.show_hop_ids(&trace, ttl)?;

        Ok(hop_ids)
    }

    fn insert_stats(&self, trace: &Uuid, ttl: u8, stats: &HopStats) -> Result<()> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/insert-stats.sql"))?;

        let hop_ids = self.show_hop_ids(&trace, ttl)?;

        for id in hop_ids {
            stmt.execute(params![
                id,
                stats.mean.map(|ms| ms.as_millis().to_string()),
                stats.median.map(|ms| ms.as_millis().to_string())
            ])?;
        }

        Ok(())
    }

    fn insert_geoip(&self, trace: &Uuid, ttl: u8, query: u8, geoip: &IpApiResp) -> Result<()> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/insert-geoip.sql"))?;

        let hop_id = self.show_hop_id(&trace, ttl, query)?;

        stmt.execute(params![
            hop_id,
            geoip.city,
            geoip.region,
            geoip.region_code,
            geoip.country_name,
            geoip.country_code,
            geoip.country_code_iso3,
            geoip.country_capital,
            geoip.latitude,
            geoip.longitude,
            geoip.timezone,
            geoip.utc_offset,
            geoip.asn,
            geoip.org,
        ])?;

        Ok(())
    }

    fn export_route(&self, source: &Ipv4Addr, destination: &Ipv4Addr) -> Result<Vec<ExportHop>> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/export-route.sql"))?;
        let columns = columns_from_statement(&stmt);

        let rows = stmt.query_and_then(
            params![&source.to_string(), &destination.to_string()],
            |row| from_row_with_columns::<ExportHop>(row, &columns),
        )?;

        let mut exports: Vec<ExportHop> = vec![];

        for row in rows {
            exports.push(row?);
        }

        Ok(exports)
    }

    fn show_geoip_for_addr(&self, source: &Ipv4Addr) -> Result<IpApiResp> {
        let conn = &self.db.connection;
        let mut stmt = conn.prepare_cached(include_str!("sql/show-geoip-for-hop.sql"))?;

        let result = stmt.query_row(params![source.to_string()], |row| {
            Ok(IpApiResp {
                ip: source.clone(),
                city: row.get(0).ok(),
                region: row.get(1).ok(),
                region_code: row.get(2).ok(),
                country_name: row.get(3).ok(),
                country_code: row.get(4).ok(),
                country_code_iso3: row.get(5).ok(),
                country_capital: row.get(6).ok(),
                latitude: row.get(7).ok(),
                longitude: row.get(8).ok(),
                timezone: row.get(9).ok(),
                utc_offset: row.get(10).ok(),
                asn: row.get(11).ok(),
                org: row.get(12).ok(),
                country_tld: None,
                country_calling_code: None,
                country_population: None,
                country_area: None,
                continent_code: None,
                in_eu: None,
                postal: None,
                currency: None,
                currency_name: None,
                languages: None,
            })
        })?;

        Ok(result)
    }
}

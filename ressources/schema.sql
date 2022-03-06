CREATE TABLE IF NOT EXISTS route (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source TEXT NOT NULL,
  destination TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_route ON route (source, destination);

CREATE TABLE IF NOT EXISTS hop (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ttl INTEGER NOT NULL,
  query INTEGER NOT NULL,
  query_result TEXT,
  addr TEXT,
  rtt TEXT,
  trace INTEGER NOT NULL REFERENCES trace(id)
);
CREATE INDEX IF NOT EXISTS idx_hop_trace ON hop (trace);

CREATE TABLE IF NOT EXISTS trace (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trace TEXT NOT NULL,
  route INTEGER NOT NULL REFERENCES route(id)
);
CREATE INDEX IF NOT EXISTS idx_trace ON trace (trace);

CREATE TABLE IF NOT EXISTS hop_stats (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hop INTEGER NOT NULL REFERENCES hop(id),
  mean_ms TEXT,
  median_ms TEXT
);
CREATE INDEX IF NOT EXISTS idx_hop_stats ON hop_stats (hop);

CREATE TABLE IF NOT EXISTS hop_geo (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hop INTEGER NOT NULL REFERENCES hop(id),
  city TEXT,
  region TEXT,
  region_code TEXT,
  country TEXT,
  country_code TEXT,
  country_code_iso3 TEXT,
  country_capital TEXT,
  latitude TEXT,
  longitude TEXT,
  timezone TEXT,
  utc_offset TEXT,
  asn TEXT,
  org TEXT
);

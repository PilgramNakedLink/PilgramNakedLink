INSERT INTO hop (
  ttl,
  trace,
  query,
  query_result,
  addr,
  rtt
) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT DO NOTHING;

SELECT
  r.source,
  r.destination,
  t.trace,
  h.ttl,
  h.query,
  h.query_result,
  h.addr,
  h.rtt,
  hs.mean_ms AS hop_mean_ms,
  hs.median_ms AS hop_median_ms,
  hg.city,
  hg.region,
  hg.region_code,
  hg.country,
  hg.country_code,
  hg.country_code_iso3,
  hg.country_capital,
  hg.latitude,
  hg.longitude,
  hg.timezone,
  hg.utc_offset,
  hg.asn,
  hg.org
FROM hop h
  JOIN trace t ON h.trace = t.id
  JOIN route r ON t.route = r.id
  LEFT JOIN hop_stats hs ON h.id = hs.hop
  LEFT JOIN hop_geo hg ON h.id = hg.hop
WHERE r.source = ?1
  AND r.destination = ?2
ORDER BY t.trace, h.ttl;

INSERT INTO hop_stats (
  hop,
  mean_ms,
  median_ms
) VALUES (?1, ?2, ?3)
ON CONFLICT DO NOTHING;

INSERT INTO trace (
  trace,
  route
) VALUES (?1, ?2)
ON CONFLICT DO NOTHING;

INSERT INTO route (
  source,
  destination
) VALUES (?1, ?2)
ON CONFLICT DO NOTHING;

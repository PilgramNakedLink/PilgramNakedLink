SELECT
  id,
  source,
  destination
FROM
  route
WHERE source = ?1
  AND destination = ?2;

SELECT
  id
FROM
  hop
WHERE ttl = ?1
  AND query = ?2
  AND trace = ?3;

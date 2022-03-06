SELECT
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
 FROM hop_geo AS hg
 JOIN hop AS h ON hg.hop = h.id
WHERE h.addr = ?1
LIMIT 1;

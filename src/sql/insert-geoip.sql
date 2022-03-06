INSERT INTO hop_geo (
  hop,
  city,
  region,
  region_code,
  country,
  country_code,
  country_code_iso3,
  country_capital,
  latitude,
  longitude,
  timezone,
  utc_offset,
  asn,
  org
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
ON CONFLICT DO NOTHING;

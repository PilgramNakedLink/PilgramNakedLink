use anyhow::{Error, Result};
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Deserialize)]
pub struct IpApiResp {
    pub ip: Ipv4Addr,
    pub city: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub country_code: Option<String>,
    pub country_code_iso3: Option<String>,
    pub country_name: Option<String>,
    pub country_capital: Option<String>,
    pub country_tld: Option<String>,
    pub country_calling_code: Option<String>,
    pub country_population: Option<f64>,
    pub country_area: Option<f64>,
    pub continent_code: Option<String>,
    pub in_eu: Option<bool>,
    pub postal: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub utc_offset: Option<String>,
    pub currency: Option<String>,
    pub currency_name: Option<String>,
    pub languages: Option<String>,
    pub asn: Option<String>,
    pub org: Option<String>,
}

pub(crate) fn fetch_ip_api(ip: &IpAddr) -> Result<IpApiResp> {
    let api_key = match env::var("TRACER_IPAPI_KEY") {
        Ok(val) => val,
        Err(_) => return Err(Error::msg("Set the TRACER_IPAPI_KEY environment variable.")),
    };

    let url = format!("https://ipapi.co/{}/json/?key={}", ip.to_string(), api_key);
    let resp = ureq::get(url.as_str()).call()?.into_json()?;

    Ok(resp)
}

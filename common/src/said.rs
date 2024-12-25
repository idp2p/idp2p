use std::str::FromStr;

use cid::Cid;
use regex::Regex;

pub struct Idp2pId {
    version: String,
    cid: Cid
}

pub fn parse_said<T>(prefix: &str, id: &str) -> Result<(String, Cid), anyhow::Error> {
    let re_str = format!("/idp2p/{prefix}/(?<major>([0-9]+))/(?<minor>([0-9]+))/(?<cid>(.+))$");
    let re = Regex::new(&re_str).unwrap();
    let Some(caps) = re.captures(id) else {
          anyhow::bail!("");
    };
    let cid = Cid::try_from(caps["cid"].to_string().as_str()).map_err(anyhow::Error::msg)?;
    let version = format!("{}.{}", caps["major"].to_string(), caps["minor"].to_string());
    Ok((version, cid))
}


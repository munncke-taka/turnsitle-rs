use std::net::IpAddr;

/// An IPv4 or IPv6 network, e.g. 10.0.0.0/8 or 2001:db8::/32. A bare IP means /32 or /128.
#[derive(Clone, Copy, Debug)]
pub struct Cidr {
    net: IpAddr,
    len: u8,
}

impl Cidr {
    pub fn parse(s: &str) -> Option<Cidr> {
        let (addr, len) = match s.split_once('/') {
            Some((a, l)) => (a, Some(l)),
            None => (s, None),
        };
        let net: IpAddr = addr.trim().parse().ok()?;
        let max = if net.is_ipv4() { 32 } else { 128 };
        let len = match len {
            Some(l) => l.trim().parse::<u8>().ok()?,
            None => max,
        };
        (len <= max).then_some(Cidr { net, len })
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        // to_canonical() turns IPv4-mapped IPv6 (::ffff:1.2.3.4) into plain IPv4
        match (self.net, ip.to_canonical()) {
            (IpAddr::V4(n), IpAddr::V4(i)) => {
                let m = u32::MAX.checked_shl(32 - self.len as u32).unwrap_or(0);
                u32::from(n) & m == u32::from(i) & m
            }
            (IpAddr::V6(n), IpAddr::V6(i)) => {
                let m = u128::MAX.checked_shl(128 - self.len as u32).unwrap_or(0);
                u128::from(n) & m == u128::from(i) & m
            }
            _ => false,
        }
    }
}

/// ALLOW_CIDRS="10.0.0.0/8, 192.168.1.0/24 2001:db8::/32" (comma and/or space separated).
/// Invalid entries abort startup so a typo never silently disables the allowlist.
pub fn parse_allowlist(raw: &str) -> Vec<Cidr> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| Cidr::parse(s).unwrap_or_else(|| panic!("invalid CIDR in ALLOW_CIDRS: {s:?}")))
        .collect()
}

extern crate hyper;

use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use iron::headers::parsing::from_one_raw_str;
use iron::headers::{Header, HeaderFormat};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct XForwardedFor {
    pub ip_address: IpAddr,
    pub proxy_ips: Vec<IpAddr>,
}

impl Header for XForwardedFor {
    fn header_name() -> &'static str {
        "X-Forwarded-For"
    }

    fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<XForwardedFor> {
        from_one_raw_str(raw)
    }
}

impl HeaderFormat for XForwardedFor {
    fn fmt_header(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl FromStr for XForwardedFor {
    type Err = hyper::Error;

    fn from_str(str: &str) -> hyper::Result<XForwardedFor> {
        let mut ip_addrs: Vec<IpAddr> = Vec::new();

        for ip_str in str.split(",").map(str::trim) {
            match ip_str.parse() {
                Ok(ip) => ip_addrs.push(ip),
                Err(_) => return Err(hyper::error::Error::Header),
            }
        }

        match ip_addrs.split_first() {
            Some((ip_addr, proxy_ips)) => {
                Ok(XForwardedFor {
                    ip_address: ip_addr.clone(),
                    proxy_ips: proxy_ips.to_vec(),
                })
            }
            None => Err(hyper::error::Error::Header),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use super::XForwardedFor;

    #[derive(PartialEq, Eq, Debug)]
    enum Is {
        Valid,
        Invalid,
    }

    fn parse_ip(ip_str: &str) -> Option<IpAddr> {
        ip_str.trim().parse().ok()
    }

    fn test_fromstr_xforwardedfor(test_is: Is, str: &str) {
        let ip_list: Vec<IpAddr> = str.split(",").filter_map(parse_ip).collect();
        let parsed: XForwardedFor = match str.parse() {
            Ok(result) => result,
            Err(_) => {
                assert_eq!(test_is, Is::Invalid);
                return;
            }
        };
        let header = XForwardedFor {
            ip_address: ip_list[0],
            proxy_ips: ip_list[1..].to_vec(),
        };
        assert_eq!(test_is, Is::Valid); // We should get to this point only with valid data
        assert_eq!(parsed, header);
    }

    #[test]
    fn test_fromstr_xforwardedfor_one_ip4() {
        test_fromstr_xforwardedfor(Is::Valid, "192.0.2.25")
    }

    #[test]
    fn test_fromstr_xforwardedfor_one_ip6() {
        test_fromstr_xforwardedfor(Is::Valid, "2001:db8::c0:ff:ee")
    }

    #[test]
    fn test_fromstr_xforwardedfor_many_ips() {
        test_fromstr_xforwardedfor(Is::Valid, "198.51.100.255, 2001:db8::CAFE, 203.0.113.42")
    }

    #[test]
    fn test_fromstr_xforwardedfor_hostname() {
        test_fromstr_xforwardedfor(Is::Invalid, "dotdot.example.com")
    }

    #[test]
    fn test_fromstr_xforwardedfor_empty() {
        test_fromstr_xforwardedfor(Is::Invalid, "")
    }

}

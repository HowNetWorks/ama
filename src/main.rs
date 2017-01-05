mod json_result;
mod x_forwarded_for;

extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate cymrust;
extern crate resolve;

use iron::headers::{Headers, ContentType, CacheControl, CacheDirective};
use iron::modifiers::Header;

use iron::prelude::*;
use iron::status;
use router::Router;

use rustc_serialize::Encodable;

use std::time::{Duration, SystemTime};
use std::net::IpAddr;
use std::cmp::min;

use cymrust::*;
use resolve::resolver::resolve_addr;

use json_result::{as_json_result, as_json_error};
use x_forwarded_for::XForwardedFor;


#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, RustcEncodable)]
struct IPInfo {
    ip: String,
    name: Option<String>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, RustcEncodable)]
struct Cymru {
    pub ip_addr: String,
    pub bgp_prefix: String,
    pub as_number: u32,
    pub as_name: String,
    pub country_code: String,
    pub registry: String,
    pub allocated: Option<String>,
}

enum Cache {
    Public(Duration),
    NoCache
}


fn main() {
    let listen_on = "0:8080";
    let mut router = Router::new();

    // Google's health check requires 200 OK response from root
    router.get("/", ok_handler, "ok_handler");

    router.get("/api/cymru/:ip", cymru_handler, "cymru_handler");
    router.get("/api/reverse/:ip", reverse_handler, "reverse_handler");
    router.get("/api/whoami/", whoami_handler, "whoami_handler");

    println!("Listening on {}", listen_on);
    Iron::new(router).http(listen_on).unwrap();
}


fn ok_handler(_: &mut Request) -> IronResult<Response> {
    let response = Response::with((status::Ok, "ok\n"));
    Ok(response)
}


fn cymru_handler(request: &mut Request) -> IronResult<Response> {
    let router = request.extensions.get::<Router>().unwrap();
    let ip_str = router.find("ip").unwrap();

    let ip: IpAddr = match ip_str.parse() {
        Err(err) => return badreq_response(format!("{}", err)),
        Ok(val) => val,
    };

    fn to_encodable(ip2asn: CymruIP2ASN) -> Cymru {
        Cymru {
            ip_addr: ip2asn.ip_addr.to_string(),
            bgp_prefix: ip2asn.bgp_prefix,
            as_number: ip2asn.as_number,
            as_name: ip2asn.as_name,
            country_code: ip2asn.country_code,
            registry: ip2asn.registry,
            allocated: ip2asn.allocated,
        }
    }

    match cymru_ip2asn(ip) {
        Err(err) => badreq_response(err),
        Ok(ip2asn) => {
            let mut results: Vec<Cymru> = Vec::with_capacity(ip2asn.len());

            let now = SystemTime::now();
            let mut max_age = Duration::from_secs(365*24*60*60);

            for item in ip2asn {
                let expires = item.expires;
                let encodable = to_encodable(item);
                results.push(encodable);

                match expires.duration_since(now) {
                    Err(_) => continue,
                    Ok(duration) => if duration < max_age {
                        max_age = duration;
                    }
                }
            }
            ok_response(results, Cache::Public(max_age))
        }
    }
}


fn reverse_handler(request: &mut Request) -> IronResult<Response> {
    let router = request.extensions.get::<Router>().unwrap();
    let ip_str = router.find("ip").unwrap();
    let max_age = Duration::from_secs(30); // FIXME: This should come from DNS TTL

    match ip_str.parse::<IpAddr>() {
        Err(err) => badreq_response(format!("{}", err)),
        Ok(ip) => ok_response(reverse_lookup(ip), Cache::Public(max_age))
    }
}


fn whoami_handler(request: &mut Request) -> IronResult<Response> {
    let ip: IpAddr = match request.headers.get::<XForwardedFor>() {
        Some(res) => res.ip_address,
        None => request.remote_addr.ip(),
    };

    ok_response(reverse_lookup(ip), Cache::NoCache)
}


fn reverse_lookup(ip: IpAddr) -> IPInfo {
    let addr_str = format!("{}", ip);
    let name_str = resolve_addr(&ip).ok();

    IPInfo {
        ip: addr_str,
        name: name_str,
    }
}

fn ok_response<T: Encodable>(result: T, cache: Cache) -> IronResult<Response> {
    let encoded = as_json_result(result);
    let mut headers = Headers::new();
    let mut response = Response::with((status::Ok, encoded));

    let max = 365*24*60*60; // year in seconds
    let cache_control = match cache {
        Cache::Public(max_age) => {
            let seconds = min(max_age.as_secs(), max) as u32;
            vec![CacheDirective::MaxAge(seconds), CacheDirective::Public]
        },
        Cache::NoCache => {
            vec![CacheDirective::NoCache, CacheDirective::NoStore, CacheDirective::MustRevalidate]
        }
    };

    headers.set(ContentType::json());
    headers.set(CacheControl(cache_control));
    response.headers = headers;

    Ok(response)
}

fn badreq_response<T: Encodable>(err: T) -> IronResult<Response> {
    let json = as_json_error(err);
    let response = Response::with((status::BadRequest, Header(ContentType::json()), json));
    Ok(response)
}

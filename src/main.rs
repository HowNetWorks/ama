mod json_result;
mod x_forwarded_for;

extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate cymrust;
extern crate resolve;

use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use router::Router;

use rustc_serialize::Encodable;

use std::net::IpAddr;

use cymrust::*;
use resolve::resolver::resolve_addr;

use json_result::{as_json_result, as_json_error};
use x_forwarded_for::XForwardedFor;


#[derive(RustcEncodable, Debug)]
struct IPInfo {
    ip: String,
    name: Option<String>,
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

    match cymru_ip2asn(ip) {
        Err(err) => badreq_response(err),
        Ok(ip2asn) => ok_response(ip2asn),
    }
}


fn reverse_handler(request: &mut Request) -> IronResult<Response> {
    let router = request.extensions.get::<Router>().unwrap();
    let ip_str = router.find("ip").unwrap();

    match ip_str.parse::<IpAddr>() {
        Err(err) => badreq_response(format!("{}", err)),
        Ok(ip) => ok_response(reverse_lookup(ip))
    }
}


fn whoami_handler(request: &mut Request) -> IronResult<Response> {
    let ip: IpAddr = match request.headers.get::<XForwardedFor>() {
        Some(res) => res.ip_address,
        None => request.remote_addr.ip(),
    };

    ok_response(reverse_lookup(ip))
}


fn reverse_lookup(ip: IpAddr) -> IPInfo {
    let addr_str = format!("{}", ip);
    let name_str = resolve_addr(&ip).ok();

    IPInfo {
        ip: addr_str,
        name: name_str,
    }
}

fn ok_response<T: Encodable>(result: T) -> IronResult<Response> {
    let encoded = as_json_result(result);
    let response = Response::with((status::Ok, Header(ContentType::json()), encoded));
    Ok(response)
}

fn badreq_response<T: Encodable>(err: T) -> IronResult<Response> {
    let json = as_json_error(err);
    let response = Response::with((status::BadRequest, Header(ContentType::json()), json));
    Ok(response)
}

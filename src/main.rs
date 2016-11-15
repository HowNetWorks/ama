mod json_result;
mod x_forwarded_for;

extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate cymrust;
extern crate resolve;

use iron::prelude::*;
use iron::status;
use router::Router;

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

    /* Google's health check requires 200 OK response from root */
    router.get("/", ok_handler, "ok_handler");

    router.get("/ama/cymru/:ip", cymru_handler, "cymru_handler");
    router.get("/ama/reverse/:ip", reverse_handler, "reverse_handler");
    router.get("/ama/whoami/", whoami_handler, "whoami_handler");

    println!("Listening on {}", listen_on);
    Iron::new(router).http(listen_on).unwrap();
}


fn ok_handler(_: &mut Request) -> IronResult<Response> {
    let response = Response::with((status::Ok, "ok"));
    Ok(response)
}


fn cymru_handler(request: &mut Request) -> IronResult<Response> {
    let router = request.extensions.get::<Router>().unwrap();
    let ip_str = router.find("ip").unwrap();

    let ip: IpAddr = match ip_str.parse() {
        Err(err) => {
            let json = as_json_error(format!("{}", err));
            let response = Response::with((status::BadRequest, json));
            return Ok(response);
        }
        Ok(val) => val,
    };

    let ip2asn: Vec<CymruIP2ASN> = match cymru_ip2asn(ip) {
        Err(err) => {
            let json = as_json_error(err);
            let response = Response::with((status::BadRequest, json));
            return Ok(response);
        }
        Ok(val) => val,
    };

    let encoded = as_json_result(&ip2asn);
    let response = Response::with((status::Ok, encoded));

    Ok(response)
}


fn reverse_handler(request: &mut Request) -> IronResult<Response> {
    let router = request.extensions.get::<Router>().unwrap();
    let ip_str = router.find("ip").unwrap();

    let ip: IpAddr = match ip_str.parse() {
        Err(err) => {
            let json = as_json_error(format!("{}", err));
            let response = Response::with((status::BadRequest, json));
            return Ok(response);
        }
        Ok(val) => val,
    };

    let addr_str = format!("{}", ip);
    let name_str = resolve_addr(&ip).ok();

    let ip_info = IPInfo {
        ip: addr_str,
        name: name_str,
    };

    let encoded = as_json_result(ip_info);
    let response = Response::with((status::Ok, format!("{}\n", encoded)));

    Ok(response)
}


fn whoami_handler(request: &mut Request) -> IronResult<Response> {
    let ip: IpAddr = match request.headers.get::<XForwardedFor>() {
        Some(res) => res.ip_address,
        None => request.remote_addr.ip(),
    };

    let addr_str = format!("{}", ip);
    let name_str = resolve_addr(&ip).ok();

    let ip_info = IPInfo {
        ip: addr_str,
        name: name_str,
    };

    let encoded = as_json_result(ip_info);
    let response = Response::with((status::Ok, format!("{}\n", encoded)));

    Ok(response)
}

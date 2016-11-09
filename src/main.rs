extern crate iron;
extern crate router;
extern crate cymrust;
extern crate rustc_serialize;

use std::net::IpAddr;

use iron::prelude::*;
use iron::status;
use router::Router;

use rustc_serialize::json;

use cymrust::*;


fn main() {
    let listen_on = "0:8080";
    let mut router = Router::new();

    /* Google's health check requires 200 OK response from root */
    router.get("/", ok_handler, "ok_handler");

    /* Handle IP addresses from root level */
    router.get("/:ip", cymru_handler, "cymru_handler1");

    /* And under /ama-cymru/. Needed for GKE's Ingress */
    router.get("/ama-cymru/:ip", cymru_handler, "cymru_handler2");

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
            let response = Response::with((status::BadRequest, format!("{}\n", err)));
            return Ok(response);
        }
        Ok(val) => val,
    };

    let ip2asn: Vec<CymruIP2ASN> = match cymru_ip2asn(ip) {
        Err(err) => {
            let response = Response::with((status::NotFound, format!("{}\n", err)));
            return Ok(response);
        }
        Ok(val) => val,
    };

    let encoded = json::as_pretty_json(&ip2asn);
    let response = Response::with((status::Ok, format!("{}\n", encoded)));

    Ok(response)
}

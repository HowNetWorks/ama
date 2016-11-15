extern crate rustc_serialize;

use rustc_serialize::Encodable;
use rustc_serialize::json;

#[derive(RustcEncodable, Debug)]
struct JSONResult<T> {
    data: T,
}

#[derive(RustcEncodable, Debug)]
struct JSONError<T> {
    errors: T,
}

pub fn as_json_result<T: Encodable>(result: T) -> String {
    let json = JSONResult { data: result };
    let pretty = json::as_pretty_json(&json);
    format!("{}", pretty)
}

pub fn as_json_error<T: Encodable>(result: T) -> String {
    let json = JSONError { errors: [ result ] };
    let pretty = json::as_pretty_json(&json);
    format!("{}", pretty)
}

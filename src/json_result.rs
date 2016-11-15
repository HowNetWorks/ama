extern crate rustc_serialize;

use rustc_serialize::Encodable;
use rustc_serialize::json;

#[derive(RustcEncodable, Debug)]
struct JSONResult<T> {
    data: T,
}

#[derive(RustcEncodable, Debug)]
struct JSONError<T> {
    message: T,
}

pub fn as_json_result<T: Encodable>(result: T) -> String {
    let json = JSONResult { data: result };
    let pretty = json::as_pretty_json(&json);
    format!("{}\n", pretty)
}

pub fn as_json_error<T: Encodable>(error_msg: T) -> String {
    let json = JSONError { message: error_msg };
    let pretty = json::as_pretty_json(&json);
    format!("{}\n", pretty)
}

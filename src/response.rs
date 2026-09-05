use std::{fs, io::Write, net::TcpStream};

use http::{Response, StatusCode, Version};

use crate::serializer::serialize;

pub fn from_string(string: String, status: StatusCode) -> Response<String> {
    Response::builder()
        .version(Version::HTTP_11)
        .header("content-length", string.len())
        .status(status)
        .body(string)
        .unwrap()
}

pub fn render_file(path: String, status: StatusCode) -> Response<String> {
    let contents = fs::read_to_string(path);
    from_string(contents.unwrap(), status)
}

pub fn write_response<T: Into<Vec<u8>>>(mut stream: &TcpStream, response: Response<T>) {
    let bytes = serialize(response).unwrap();
    stream.write_all(&mut bytes.as_slice()).unwrap();
}

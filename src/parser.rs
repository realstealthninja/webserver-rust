use std::{
    error::Error,
    fmt,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use http::{HeaderName, HeaderValue, Request, Version, header::CONTENT_LENGTH};

#[derive(Debug)]
pub struct HttpParseError {}

impl fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse Http Request")
    }
}

impl Error for HttpParseError {}

pub fn parse(stream: &TcpStream) -> Result<Request<String>, HttpParseError> {
    let mut buffer = BufReader::new(stream);

    let headers: Vec<_> = buffer
        .by_ref()
        .lines()
        .map(|line| line.unwrap_or_default())
        .take_while(|line| !line.is_empty())
        .collect();

    let start_line = match headers.first().clone() {
        Some(start_line) => start_line,
        None => return Err(HttpParseError {}),
    };

    let mut request = Request::default();
    // // parse header
    let start_line = start_line.split(" ").collect::<Vec<_>>();

    *request.method_mut() = start_line[0].parse().unwrap();

    *request.uri_mut() = start_line[1].parse().unwrap();

    *request.version_mut() = match start_line[2] {
        "HTTP/0.9" => Version::HTTP_09,
        "HTTP/1.0" => Version::HTTP_10,
        "HTTP/1.1" => Version::HTTP_11,
        "HTTP/2.0" => Version::HTTP_2,
        "HTTP/3.0" => Version::HTTP_3,
        _ => return Err(HttpParseError {}),
    };

    for line in headers[1..].into_iter() {
        let header = line.split_once(':').unwrap();

        let header_name: HeaderName = header.0.trim().parse().unwrap();
        let header_value: HeaderValue = header.1.trim().parse().unwrap();

        request.headers_mut().insert(header_name, header_value);
    }

    match request.headers().get(CONTENT_LENGTH) {
        Some(length) => {
            let _ = buffer
                .take(length.to_str().unwrap().parse().unwrap())
                .read_to_string(request.body_mut());
        }
        None => {}
    };

    Ok(request)
}

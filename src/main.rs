use clap::Parser;
use core::fmt;
use http::{
    HeaderName, HeaderValue, Request, Response, StatusCode, Version, header::CONTENT_LENGTH,
};
use log;
use webserver::ThreadPool;

use std::{
    error::Error,
    fmt::Debug,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    vec,
};

#[derive(Debug)]
struct HttpParseError {}

#[derive(Debug)]
struct HttpSerializeError {}

impl fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse Http Request")
    }
}

impl Error for HttpParseError {}

impl fmt::Display for HttpSerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to serialize Http response")
    }
}

impl Error for HttpSerializeError {}

fn serialize<T: Into<Vec<u8>>>(response: Response<T>) -> Result<Vec<u8>, HttpSerializeError> {
    let (parts, body) = response.into_parts();
    let version = match parts.version {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        Version::HTTP_3 => "HTTP/3.0",
        _ => return Err(HttpSerializeError {}),
    };

    let status_code = parts.status.as_u16();

    let start_line = format!(
        "{} {} {}",
        version,
        status_code,
        parts.status.canonical_reason().unwrap()
    );

    let mut headers: Vec<String> = vec![];
    for header in parts.headers {
        headers.push(format!(
            "{}: {}",
            match header.0 {
                Some(x) => x.to_string(),
                _ => return Err(HttpSerializeError {}),
            },
            header.1.to_str().unwrap()
        ));
    }

    let message = format!("{}\n{}\n\n", start_line, headers.join("\n"),);

    let mut message: Vec<u8> = message.into();
    let mut body: Vec<u8> = body.into();

    message.append(&mut body);

    Ok(message)
}

fn parse(stream: &TcpStream) -> Result<Request<String>, HttpParseError> {
    let mut buffer = BufReader::new(stream);

    let headers: Vec<_> = buffer
        .by_ref()
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let start_line = headers.first().clone().unwrap();

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

fn string_response(string: String, status: StatusCode) -> Response<String> {
    Response::builder()
        .version(Version::HTTP_11)
        .header("content-length", string.len())
        .status(status)
        .body(string)
        .unwrap()
}

fn render_file(path: String, status: StatusCode) -> Response<String> {
    let contents = fs::read_to_string(path);
    string_response(contents.unwrap(), status)
}

fn sleep(_: Request<String>) -> Response<String> {
    thread::sleep(Duration::from_secs(5));
    return string_response(
        "waited 5 seconds".to_owned(),
        StatusCode::from_u16(200).unwrap(),
    );
}

fn index(_: Request<String>) -> Response<String> {
    return render_file(
        "templates/index.html".to_owned(),
        StatusCode::from_u16(200).unwrap(),
    );
}

fn handle_connection(mut stream: TcpStream) {
    let request = parse(&stream).unwrap();
    println!("handling {}", *request.uri());

    let response = match *&request.uri().path() {
        "/" => index(request),
        "/sleep" => sleep(request),
        _ => Response::builder()
            .version(Version::HTTP_11)
            .header("Content-Length", 0)
            .status(404)
            .body("".to_string())
            .unwrap(),
    };
    let response = serialize(response).unwrap();

    stream.write_all(&mut response.as_slice()).unwrap();
    stream.flush().unwrap();
}
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    thread_count: usize,
}

fn main() {
    let args = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(args.thread_count);

    for stream in listener.incoming() {
        pool.execute(|| {
            match stream {
                Ok(stream) => handle_connection(stream),
                Err(err) => log::error!("Failed to accept connection: {}", err),
            };
        });
    }
}

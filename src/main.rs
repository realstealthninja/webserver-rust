use clap::Parser;
use http::{Request, Response, StatusCode, Version};
use log::{self, error, info};
use std::{
    error::Error,
    fmt::Debug,
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use webserver::{
    ThreadPool,
    parser::parse,
    response::{self, render_file, write_response},
};

fn sleep(_: Request<String>) -> Response<String> {
    thread::sleep(Duration::from_secs(5));
    return response::from_string(
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
    let request = match parse(&stream) {
        Ok(request) => request,
        Err(_) => {
            error!("Could not parse request");
            write_response(
                &stream,
                Response::builder()
                    .version(Version::HTTP_11)
                    .status(500)
                    .body("".to_string())
                    .unwrap(),
            );
            return;
        }
    };

    info!("handling {}", *request.uri());

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

    write_response(&stream, response);
    stream.flush().unwrap();
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    thread_count: usize,

    #[arg(short, long, default_value_t = 7878)]
    port: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let args = Args::parse();
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", args.port)) {
        Ok(listener) => listener,
        Err(err) => {
            error!("Failed to bind to address 127.0.0.1:{}: {}", args.port, err);
            return Err(Box::new(err));
        }
    };

    let pool = ThreadPool::new(args.thread_count);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool.execute(|| handle_connection(stream)),
            Err(err) => error!("Failed to accept connection: {}", err),
        };
    }

    Ok(())
}

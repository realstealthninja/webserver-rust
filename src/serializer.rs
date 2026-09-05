use std::{error::Error, fmt};

use http::{Response, Version};

#[derive(Debug)]
pub struct HttpSerializeError {}

impl fmt::Display for HttpSerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to serialize Http response")
    }
}

impl Error for HttpSerializeError {}

pub fn serialize<T: Into<Vec<u8>>>(response: Response<T>) -> Result<Vec<u8>, HttpSerializeError> {
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

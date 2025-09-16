use std::{io::prelude::*, net::TcpStream};

#[derive(Default)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String
}

impl HttpRequest {

    pub fn new(request_data: String) -> Self {

        if let Some((request_line, _rest)) =
            request_data.split_once("\r\n") {
                let segments: Vec<&str> = 
                    request_line.split_whitespace().collect();

                if segments.len() == 3 {
                    Self {
                        method: segments[0].to_string(),
                        uri: segments[1].to_string()
                    }
                } else {
                    Self::default()
                }
            } else {
                Self::default()
            }
    }

}

pub struct HttpResponse {
    pub stream: TcpStream
}

impl HttpResponse {
    pub fn send(&mut self, output: String) {
        self.stream.write(
            format!("HTTP/1.1 200 OK \r\n\r\n{}", output).as_bytes()
        ).unwrap();
    }

    pub fn error(&mut self, error_msg: String) {
        self.stream.write(
            format!("HTTP/1.1 404 Not Found \r\n\r\n{}", error_msg).as_bytes()
        ).unwrap();
    }
}
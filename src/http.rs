use std::{collections::HashMap, hash::Hash, io::prelude::*, net::TcpStream};

#[derive(Default)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: std::collections::HashMap<String, String>,
    pub post_body: String,
}

impl HttpRequest {

    pub fn new(request_data: String) -> Self {

        if let Some((request_line, rest)) =
            request_data.split_once("\r\n") {
                let segments: Vec<&str> = 
                    request_line.split_whitespace().collect(); 

                if segments.len() == 3 {

                    let mut headers: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                    let mut post_body = String::new();

                    Self::extract(rest, &mut headers, &mut post_body);
                    
                    Self {
                        method: segments[0].to_string(),
                        uri: segments[1].to_string(),
                        headers,
                        post_body
                    }
                } else {
                    Self::default()
                }
            } else {
                Self::default()
            }
    }

    pub fn extract(request_data: &str, headers: &mut std::collections::HashMap<String, String>, post_body: &mut String) {

        if let Some((header_line, rest)) = request_data.split_once("\r\n") {
            if header_line.is_empty() {
                post_body.push_str(rest);
            } else {
                let key_value = header_line.split_once(":").unwrap();
                headers.insert(key_value.0.to_string(), key_value.1.to_string());

                Self::extract(rest, headers, post_body);
            } 
        }
    }

    // to do
    pub fn parse_uri(uri: &str) -> (String, HashMap<String, String>) {

        if let Some((path, query_string)) = uri.split_once("?") {

            (path.to_string(), HashMap::new())
        } else {
            (uri.to_string(), HashMap::new())
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
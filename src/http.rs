use std::{collections::HashMap, io::prelude::*, net::TcpStream};

#[derive(Default)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: std::collections::HashMap<String, String>,
    pub post_body: String,
    pub query_params: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
}

impl HttpRequest {

    pub fn new(request_data: String) -> Self {

        if let Some((request_line, rest)) =
            request_data.split_once("\r\n") {
                let segments: Vec<&str> = 
                    request_line.split_whitespace().collect(); 

                if segments.len() == 3 {

                    let mut headers: HashMap<String, String> = HashMap::new();
                    let mut post_body = String::new();

                    Self::extract(rest, &mut headers, &mut post_body);
                    let (path, query_params) = Self::parse_uri(segments[1]);

                    Self {
                        method: segments[0].to_string(),
                        uri: path,
                        headers,
                        post_body,
                        query_params,
                        path_params: HashMap::new(),
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

    pub fn parse_uri(uri: &str) -> (String, HashMap<String, String>) {
        let mut h: HashMap<String, String> = HashMap::new();
        if let Some((path, query_string)) = uri.split_once("?") {
            for query in query_string.split('&') {
                if let Some((key, value)) = query.split_once('=') {
                    h.insert(key.to_string(), value.to_string());
                }
            }
            (path.to_string(), h)      
        } else {
            (uri.to_string(), h)
        }
    }

    pub fn get_path_param(&self, key: &str) -> Option<&String> {
        self.path_params.get(key)
    }

    pub fn get_path_param_as<T>(&self, key: &str) -> Option<T> 
    where
        T: std::str::FromStr
    {
       self.path_params.get(key)?.parse().ok()
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn get_query_param_as<T>(&self, key: &str) -> Option<T> 
    where
        T: std::str::FromStr
    {
       self.query_params.get(key)?.parse().ok()
    }

}

pub struct HttpResponse {
    pub stream: TcpStream
}

impl HttpResponse {
    /// Send a response with status 200 OK
    pub fn send(&mut self, output: String) {
        self.stream.write(
            format!("HTTP/1.1 200 OK \r\n\r\n{}", output).as_bytes()
        ).unwrap();
    }

    /// Send a response with status 404 Not Found
    pub fn error(&mut self, error_msg: String) {
        self.stream.write(
            format!("HTTP/1.1 404 Not Found \r\n\r\n{}", error_msg).as_bytes()
        ).unwrap();
    }
}
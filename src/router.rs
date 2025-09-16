use std::{io::prelude::*, net::{TcpListener, TcpStream}};

use crate::{HttpRequest, HttpResponse};

pub type RouteHandler = fn(HttpRequest, HttpResponse);

pub struct Router {
    host: String,
    port: u16,
    routes: Vec<Route>,
    error_handler: Option<RouteHandler>, 
}

impl Router {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(), 
            port,
            routes: Vec::new(),
            error_handler: None,
        }
    }


    pub fn get(&mut self, route: &str, handler: RouteHandler) {
        let route = Route {
            method: "GET".to_string(),
            route: route.to_string(),
            handler
        };

        self.routes.push(route);
    }

    pub fn post(&mut self, route: &str, handler: RouteHandler) {
    let route = Route {
        method: "POST".to_string(),
        route: route.to_string(),
        handler
    };

    self.routes.push(route);
    
    }

    pub fn error (&mut self, handler: RouteHandler) {
        self.error_handler = Some(handler);
    }

    pub fn serve(self) {
        let listener = TcpListener::bind((self.host, self.port)).unwrap();

        //let _pool = ThreadPool::new(4);

        for incoming in listener.incoming() {

            match incoming {
                Ok(stream) => {
                    //pool.execute(|| {
                    Self::handle_connection(stream, &self.routes, &self.error_handler)
                    //})
                },
                Err(_err) => {

                }
            }
        }
    }

    fn handle_connection(mut stream: TcpStream, routes: &Vec<Route>, error_handler: &Option<RouteHandler>) {
        let mut buffer : [u8; 1024] = [0; 1024];

        let _size = stream.read(&mut buffer).unwrap();

        let data = std::str::from_utf8(&buffer).unwrap();

        let request = HttpRequest::new(data.to_string());



        let matching_route = routes.iter().find(|route| {
            route.method == request.method && route.route == request.uri
        });

        if let Some(route) = matching_route {
            let response = HttpResponse {stream: stream};
            (route.handler)(request, response);

        } else {
            let mut response = HttpResponse {stream: stream};

            if let Some(handler) = error_handler {
                (handler)(request, response);
            } else {
                response.send("404 - Page not found".to_string())
            }

        }
    }
}


pub struct Route {
    method: String,
    route: String,
    handler: RouteHandler
}
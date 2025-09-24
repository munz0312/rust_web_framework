use std::{io::prelude::*, net::{TcpListener, TcpStream}, sync::Arc};

use crate::{HttpRequest, HttpResponse, ThreadPool};

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
        let path_param = Self::parse_path_param(route);
        let route = Route {
            method: "GET".to_string(),
            route: route.to_string(),
            handler,
            path_param
        };

        self.routes.push(route);
    }

    pub fn post(&mut self, route: &str, handler: RouteHandler) {
        let path_param = Self::parse_path_param(route);
        let route = Route {
            method: "POST".to_string(),
            route: route.to_string(),
            handler,
            path_param
        };

        self.routes.push(route);
    
    }

    fn parse_path_param(uri: &str) -> Option<String> {
        
        if let Some((_, param_name)) = uri.split_once(':') {
            println!("{}", param_name);
            Some(param_name.to_string())
        } else {
            None
        }
    }

    pub fn error (&mut self, handler: RouteHandler) {
        self.error_handler = Some(handler);
    }

    pub fn serve(self) {
        let listener = TcpListener::bind((self.host, self.port)).unwrap();

        let pool = ThreadPool::new(4);

        let routes_arc = Arc::new(self.routes);
        let error_arc = Arc::new(self.error_handler);

        for incoming in listener.incoming() {

            match incoming {
                Ok(stream) => {

                    let routes_clone = routes_arc.clone();
                    let error_clone = error_arc.clone();

                    pool.execute(|| {
                        Self::handle_connection(stream, routes_clone, error_clone)
                    })
                },
                
                Err(_err) => {

                }
            }
        }
    }

    fn handle_connection(mut stream: TcpStream, routes: Arc<Vec<Route>>, error_handler: Arc<Option<RouteHandler>>) {
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

            if let Some(handler) = error_handler.as_ref() {
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
    handler: RouteHandler,
    path_param: Option<String>
}
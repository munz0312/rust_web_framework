use std::{collections::HashMap, io::prelude::*, net::{TcpListener, TcpStream}, sync::Arc};

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

    /// Defines a GET route.
    /// 
    /// `route` is the URI for the route.
    /// 
    /// `handler` is the closure that will run when the route is accessed
    pub fn get(&mut self, route: &str, handler: RouteHandler) {

        let route = Route::new("GET".to_string(), route.to_string(), handler);

        self.routes.push(route);
    }

    /// Defines a POST route.
    /// 
    /// `route` is the URI for the route.
    /// 
    /// `handler` is the closure that will run when the route is accessed.
    pub fn post(&mut self, route: &str, handler: RouteHandler) {
        let route = Route::new("POST".to_string(), route.to_string(), handler);

        self.routes.push(route);
    
    }

    /// Sets the closure to run when attempting to access not found routes.
    pub fn error (&mut self, handler: RouteHandler) {
        self.error_handler = Some(handler);
    }

    /// Starts the web server!
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

        let  mut request = HttpRequest::new(data.to_string());

        let matching_route = routes.iter().find_map(|route| {
             if route.method == request.method {
                if let Some(path_params) = route.matches(&request.uri) {
                    request.path_params = path_params;
                    Some(route)
                } else {
                    None
                }
             } else {
                None
             }
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
    route_segments: Vec<RouteSegment>,
}

#[derive(Clone)]
pub enum RouteSegment {
    Static(String),
    Parameter(String),
}

impl Route {
    fn new(method: String, route: String, handler: RouteHandler) -> Self {
        let route_segments = Self::parse_route(&route);
        Route {
            method,
            route,
            handler,
            route_segments,
        }
    }

    fn parse_route(route: &str) -> Vec<RouteSegment> {
        route.split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
            if segment.starts_with(':') {
                RouteSegment::Parameter(segment[1..].to_string())
            } else {
                RouteSegment::Static(segment.to_string())
            }
        }).collect()
    }

    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<_> = path.split('/')
            .filter(|segment| !segment.is_empty())
            .collect();

        if path_segments.len() != self.route_segments.len() {
            return None
        }

        let mut params = HashMap::new();

        for (route_seg, path_seg) in self.route_segments.iter().zip(path_segments.iter()) {
            match route_seg {
                RouteSegment::Static(static_part) => {
                    if static_part != path_seg {
                        return None;
                    }
                }
                RouteSegment::Parameter(param) => {
                    params.insert(param.clone(), path_seg.to_string());
                }
            }
        }
        Some(params)
    }
}
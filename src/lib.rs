use std::{io::{prelude::*}, net::{TcpListener, TcpStream}, sync::{mpsc::{self}, Arc, Mutex}, thread};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            let worker = Worker::new(id, Arc::clone(&receiver));
            workers.push(worker);
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }

}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            worker.thread.join().unwrap();
        }
    }
}

pub struct Worker{
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing...");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });
        Worker { id, thread}
    }
}

#[derive(Default)]
pub struct HttpRequest {
    method: String,
    uri: String
}

impl HttpRequest {

    fn new(request_data: String) -> Self {

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
    stream: TcpStream
}

impl HttpResponse {
    pub fn send(&mut self, output: String) {
        self.stream.write(
            format!("HTTP/1.1 200 OK \r\n\r\n{}", output).as_bytes()
        ).unwrap();
    }
}

type RouteHandler = fn(HttpRequest, HttpResponse);

pub struct Router {
    host: String,
    port: u16,
    routes: Vec<Route>, 
}

impl Router {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(), 
            port,
            routes: Vec::new(),
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

    pub fn serve(self) {
        let listener = TcpListener::bind((self.host, self.port)).unwrap();

        let _pool = ThreadPool::new(4);

        for incoming in listener.incoming() {

            match incoming {
                Ok(stream) => {
                    //pool.execute(|| {
                    Self::handle_connection(stream, &self.routes)
                    //})
                },
                Err(_err) => {

                }
            }
        }
    }

    fn handle_connection(mut stream: TcpStream, routes: &Vec<Route>) {
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
            response.send("404 - Page not found".to_string());
        }
    }
}


pub struct Route {
    method: String,
    route: String,
    handler: RouteHandler
}
use std::{collections::HashMap, fs, io::{prelude::*, BufReader}, net::TcpStream, sync::{mpsc::{self}, Arc, Mutex}, thread};

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

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn parse(stream: &TcpStream) -> Result<HttpRequest, Box<dyn std::error::Error>> {
        let buf_reader = BufReader::new(stream);
        let request_line = buf_reader.lines().next().unwrap().unwrap();

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid request line".into());
        }

        Ok(HttpRequest {
            method: parts[0].to_string(),
            path: parts[1].to_string(),
            headers: HashMap::new(),
        })
    }
}

pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, status_text: &str, body: String) -> Self {
        HttpResponse {
            status_code: status_code,
            status_text: status_text.to_string(),
            body,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code.to_string(),
            self.status_text,
            self.body.len(),
            self.body
        )
    }
}

type RouteHandler = Box<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>;

pub struct Router {
    routes: HashMap<(String, String), RouteHandler>, 
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: HashMap::new()
        }
    }


    pub fn get<F>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.routes.insert(
            ("GET".to_string(), path.to_string()),
            Box::new(handler)
        );
        self
    }

    pub fn post<F>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.routes.insert(
            ("POST".to_string(), path.to_string()),
            Box::new(handler),
        );
        self
    }

    pub fn handle(&self, request: &HttpRequest) -> HttpResponse {
        let key = (request.method.clone(), request.path.clone());
        
        if let Some(handler) = self.routes.get(&key) {
            handler(request)
        } else {
            let body = fs::read_to_string("404.html")
                .unwrap_or_else(|_| "404 Not Found".to_string());
            HttpResponse::new(404, "NOT FOUND", body)
        }
    }

}
use std::{
    fs, io::prelude::*, net::{TcpListener, TcpStream}, sync::Arc, thread, time::Duration
};
use hello::{HttpRequest, HttpResponse, ThreadPool, Router};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    
    let pool = ThreadPool::new(4);

    let router = Arc::new(Router::new()
        .get("/", |_req| {
        let contents = fs::read_to_string("hello.html")
                .unwrap_or_else(|_| "Hello World!".to_string());
        HttpResponse::new(200, "OK", contents)
    }));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let router = Arc::clone(&router);
        
        pool.execute(move || {
            handle_connection_v2(stream, router);
        });
    }

    println!("Shutting down.")
}

fn handle_connection(mut stream: TcpStream) {
    let request = match HttpRequest::parse(&stream) {
        Ok(req) => req,
        Err(_) => {
            let response = HttpResponse::new(400, "Bad Request", "Invalid request".to_string());
            stream.write_all(response.to_string().as_bytes()).unwrap();
            return;
        }
    };

    let (status_code, status_text, filename) = match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => (200, "OK", "hello.html"),
        ("GET", "/sleep") => {
            thread::sleep(Duration::from_secs(5));
            (200, "OK", "hello.html")
        }
        (_, _) => (400, "NOT FOUND", "404.html")
    };

    let body = fs::read_to_string(filename).unwrap();

    let response = HttpResponse::new(status_code, status_text, body);

    stream.write_all(response.to_string().as_bytes()).unwrap();

}

fn handle_connection_v2(mut stream: TcpStream, router: Arc<Router>) {

    let request = match HttpRequest::parse(&stream) {
        Ok(req) => req,
        Err(_) => {
            let response = HttpResponse::new(400, "Bad Request", "Invalid request".to_string());
            stream.write_all(response.to_string().as_bytes()).unwrap();
            return}
    };

    let response = router.handle(&request);
    stream.write_all(response.to_string().as_bytes()).unwrap();
}
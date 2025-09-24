use std::fs;

use hello::Router;

fn main() {

    let mut router = Router::new("127.0.0.1", 7878);

    router.get("/home", |_req, mut res| {
        let contents = fs::read_to_string("hello.html").unwrap();
        res.send(contents);
    });

    router.post("/home", |req, mut res| {
        println!("{:?} - {}", req.headers, req.post_body);
        res.send("Home".to_string());
    });

    router.get("/user/:id", |req, res| {

    });

    router.error(|_req, mut res| {
        let contents = fs::read_to_string("404.html").unwrap();
        res.error(contents);
    });
    
    router.serve();

}


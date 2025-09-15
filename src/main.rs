use std::fs;

use hello::Router;

fn main() {

    let mut router = Router::new("127.0.0.1", 7878);

    router.get("/home", |_req, mut res| {
        let contents = fs::read_to_string("hello.html").unwrap();
        res.send(contents);
    });

    router.post("/about", |_req, mut res| {
        res.send("About".to_string());
    });

    router.error(|_req, mut res| {
        let contents = fs::read_to_string("404.html").unwrap();
        res.send(contents);
    });
    router.serve();

}


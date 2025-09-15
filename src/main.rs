use hello::Router;

fn main() {

    let mut router = Router::new("127.0.0.1", 7878);

    router.get("/home", |req, mut res| {
        res.send("Home".to_string());
    });

    router.post("/about", |req, mut res| {
        res.send("About".to_string());
    });

    router.serve();

}


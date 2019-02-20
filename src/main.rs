// logger
extern crate pretty_env_logger;
extern crate hyper;
extern crate futures;
extern crate num_cpus;

// this is for linux only
//extern crate procfs;

#[macro_use] extern crate log;

// serde [ser,deser]ializer is used to turn datastructures into some serializable format.
// https://github.com/serde-rs/serde
use serde::{Serialize, Deserialize};

// futures and hyper used for implementing a HTTP server/client
use futures::future;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, header, Request, Response, Server, Method, StatusCode};
use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::service::service_fn_ok;


// simple serializable "point" struct
#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}


// another example of a serializable, with a "sub" serializable "point" struct
#[derive(Serialize, Deserialize, Debug)]
struct Craft {
    fuel: i32,
    vel_x: i32,
    vel_y: i32,
    vel_z: i32,
    location: Point,
}


// another example struct with ser/deserializerm utilizing usize which is "arch" max bits size
#[derive(Serialize, Deserialize, Debug)]
struct Hardware {
    cpu_count: usize,
    core_count: usize,
}


// Just a simple type alias type, user in the fn:request. Box is used for heap allocation
type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;


// some static responses
static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
static NOTFOUND: &[u8] = b"Not Found";


// HTTP request router
fn request(req: Request<Body>) -> BoxFut {

    // prepare a mutable response object
    let mut response = Response::new(Body::empty());

    // perform match on http "method" and "uri" path
    match (req.method(), req.uri().path()) {
        // these match true as they are declared
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {

            *response.body_mut() = Body::from(INDEX);
        },
        (&Method::GET, "/craft") => {
            // create a instance of one of the structs, with an embedded struct
            let craft = Craft { fuel: 12, vel_x: 1, vel_y: 2, vel_z: 2, location: Point{x:10, y:22, z:9} };

            // serde_json::to_string returns a Result of either Ok or Err depending on success.
            // this match returns a Result string object on "Ok"
            match serde_json::to_string(&craft) {
                Ok(json) => {
                    println!("success {}", json);
                    *response.body_mut() = Body::from(json);
                }
                Err(e) => {
                    warn!("serializing json: {}", e);
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    *response.body_mut() = Body::from(NOTFOUND);
                }
            };
        },
         (&Method::GET, "/stats") => {
             // get number of cpus
             let logical_core_count: usize = num_cpus::get();
             let phys_cp_count: usize = num_cpus::get_physical();
             let hw = Hardware { cpu_count: phys_cp_count, core_count: logical_core_count };
             match serde_json::to_string(&hw) {
                 Ok(json) => {
                     *response.body_mut() = Body::from(json);
                 }
                 Err(e) => {
                     warn!("serializing json: {}", e);
                     *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                 }
             };
         },
        _ => {
            // any non matching route goes to 404 town
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    // finalize the response, allocating heap for it, and fire it off back to the client
    Box::new(future::ok(response))
}


fn main() {
    // Logger
    pretty_env_logger::init();

    // Point
    let craft = Craft { fuel: 12, vel_x: 1, vel_y: 2, vel_z: 2, location: Point{x:10, y:22, z:9} };

    // Convert the Point to a JSON string.
    let serialized: String = serde_json::to_string(&craft).unwrap();

    // Prints serialized = {"x":1,"y":2}
    info!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Craft = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    info!("deserialized = {:?}", deserialized);

    // setup the http service
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr)
        .serve(|| service_fn(request))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);

}

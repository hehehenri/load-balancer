use std::borrow::BorrowMut;
use std::convert::Infallible;
use std::f32::consts::E;
use std::net::SocketAddr;
use std::collections::LinkedList;
use std::sync::Arc;
use clap::{Command, Arg};
use hyper::client::{conn, connect};
use hyper::header::HOST;
use hyper::http::{HeaderValue, request};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, Service};
use hyper::{Body, Request, Response, Client, Method};
use hyper::server::Server;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let matches = Command::new("Load Balancer")
        .arg(Arg::new("server")
            .short('s')
            .long("server")
            .takes_value(true)
            .multiple_values(true)
            .help("Server addresses"))
        .arg(Arg::new("bind")
            .short('b')
            .long("bind")
            .takes_value(true)
            .help("Value that the load balancer will be binded to"))
        .get_matches();

    let bind_address = matches.value_of("bind")
        .expect("Load balancer address is needed.");

    let available_servers: LinkedList<SocketAddr> = matches.values_of("server")
        .expect("At least one server is needed")
        .into_iter().map(|address| {
            address.parse().expect("Invalid address format")
        })
        .collect();

    let available_servers = Arc::new(Mutex::new(available_servers));

    let make_service = make_service_fn(move |conn: &AddrStream| {

        let available_servers = available_servers.clone();

        let service = service_fn(move |mut request| {
            forward_request(request, available_servers.clone())
        });

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&bind_address.parse::<SocketAddr>().unwrap())
        .serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}

// TODO: Treat all those fucked up unwraps
async fn forward_request(request: Request<Body>, available_servers: Arc<Mutex<LinkedList<SocketAddr>>>) -> Result<Response<Body>, Infallible> {
    let client = Client::new();

    let mut available_servers = available_servers.lock().await;

    let server = available_servers.pop_front().unwrap();

    let request = Request::builder()
        .uri(dbg!(server.to_string()))
        .method(request.method())
        .body(Body::from("pog")).unwrap();

    let response = client.request(request).await;

    available_servers.push_back(server);

    match response {
        Ok(response) => Ok(response),
        Err(response) => panic!("WE FUCKED!!! take a look: {:#?}", response)
    }
}

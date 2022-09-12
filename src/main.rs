use std::convert::Infallible;
use std::net::SocketAddr;
use std::collections::LinkedList;
use std::sync::Arc;
use clap::{Command, Arg};
use hyper::http::uri::Parts;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Client, Uri};
use hyper::server::Server;
use hyper_tls::HttpsConnector;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppContext {
    available_servers: Arc<Mutex<LinkedList<Uri>>>,
    host: SocketAddr,
}

impl AppContext {
    pub fn from_args(args: (SocketAddr, Arc<Mutex<LinkedList<Uri>>>)) -> Self {
        let (host, available_servers) = args;
        Self {
            available_servers,
            host,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Infallible> {
    let args = parse_args();

    let app_context = AppContext::from_args(args);

    let host = app_context.host;

    let make_service = make_service_fn(move |_: &AddrStream| {
        let app_context = app_context.clone();

        let service = service_fn(move |request| {
            forward_request(request, app_context.clone())
        });

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&host)
        .serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }

    Ok(())
}

fn parse_args() -> (SocketAddr, Arc<Mutex<LinkedList<Uri>>>) {
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
            .help("Address that the load balancer will be binded to"))
        .get_matches();

    let bind_address: SocketAddr = matches.value_of("bind")
        .expect("Load balancer address is needed.")
        .parse()
        .expect("Provided invalid server address");

    let available_servers: LinkedList<Uri> = matches.values_of("server")
        .expect("At least one server is needed")
        .into_iter().map(|address| {
            let mut parts = Parts::default();

            parts.scheme = Some("http".parse().unwrap());
            parts.authority = Some(address.parse().unwrap());
            parts.path_and_query = Some("/".parse().unwrap());

            Uri::from_parts(parts).expect("Something went wrong generating the URI.")
        })
        .collect();

    (bind_address, Arc::new(Mutex::new(available_servers)))
}

#[allow(unused)]
async fn forward_request(request: Request<Body>, app_context: AppContext) -> Result<Response<Body>, Infallible> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut available_servers = app_context.available_servers.lock().await;

    let server_address = available_servers.pop_front().unwrap();
    available_servers.push_back(server_address.clone());

    println!("Forwarding request: {:?} -> {}", &request.headers().get("host").unwrap(), &server_address);

    let (mut parts, body) = request.into_parts();

    parts.uri = server_address.clone();

    let mut request = Request::from_parts(parts, body);
    let mut response = client.request(request).await;

    match response {
        Ok(response) => Ok(response),
        Err(response) => panic!("WE FUCKED!!! take a look: {:#?}", response)
    }
}

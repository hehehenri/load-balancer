use std::net::SocketAddrV4;

use clap::{Command, Arg};

fn main() {
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

    let bind: SocketAddrV4 = matches.value_of("bind").expect("Load balancer address is needed.").parse().unwrap();

    let servers: Vec<SocketAddrV4> = matches.values_of("server")
        .expect("At least one server is needed")
        .into_iter().map(|address| {
            address.parse().expect("Invalid IPv4 format")
        })
        .collect();

    println!("{:#?} {}", servers, bind);
}

use std::env;
use std::net::SocketAddr;
use std::sync::Mutex;

use lazy_static::lazy_static;
use hyper::Method;
use rusqlite::Connection;

use crate::utils::resolve_to_socket_address;


pub const INTERCEPTABLE_METHODS : [Method; 6] = [
    Method::GET, Method::HEAD, Method::POST, Method::PUT, Method::DELETE, Method::PATCH
];

lazy_static! {
    pub static ref ROCKERY_HOST: String = env::var("ROCKERY_HOST").unwrap_or_else(
        |_|"localhost".to_string()
    ).to_owned();
    
    pub static ref ROCKERY_PORT: u16 = match env::var("ROCKERY_PORT") {
        Ok(r) => r.parse().unwrap_or_else(|_|panic!("ROCKERY_PORT is not a valid port number")),
        Err(_) => 3333
    };
    
    pub static ref TARGET_HOST: String = env::var("TARGET_HOST").unwrap_or_else(
        |_| panic!("{} env variable is not set!", "TARGET_HOST")
    ).to_owned();
    
    pub static ref TARGET_PORT: u16 = match env::var("TARGET_PORT") {
        Ok(r) => r.parse().unwrap_or_else(|_|panic!("TARGET_PORT is not a valid port number")),
        Err(_) => 80
    };
    
    /**
    Global static constant, which uses `Ipv4Addr::LOCALHOST` address
    and `ROCKERY_PORT` env variable, to create `SocketAddr`.

     - `ROCKERY_HOST` defaults to `127.0.0.1`.
     - `ROCKERY_PORT` defaults to `3000`.
    **/
    pub static ref ROCKERY_SOCKET_ADDRESS: SocketAddr = resolve_to_socket_address(
        &ROCKERY_HOST, &ROCKERY_PORT
    );
    
    /**
    Global static constant, which uses `TARGET_PORT` and
    `TARGET_HOST` env variable, to create `SocketAddr`. 
    
     - `TARGET_PORT` defaults to `80`.
    **/
    pub static ref TARGET_SOCKET_ADDRESS: SocketAddr = resolve_to_socket_address(
        &TARGET_HOST, &TARGET_PORT
    );

    pub static ref SPOOF_HOST_HEADER: bool = match env::var("SPOOF_HOST_HEADER") {
        Ok(s) => match s.to_lowercase().as_ref() {
                "1" | "true" => true,
                _ => false,
        },
        Err(_) => false
    };

    pub static ref DB : Mutex<Connection> = Mutex::new(
        // Use `Connection::open("./db.db3").unwrap()` for persistent DB. Useful when debuggning
        Connection::open_in_memory().unwrap()
    );
}

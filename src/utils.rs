use std::{env::set_var, net::SocketAddr};

use futures::executor;
use tokio::net;
use hyper::{Body, Request, body::to_bytes};
use serde_json::Value;

pub fn set_env_vars(){
    //! Sets environment variables `ROCKERY_HOST`, `ROCKERY_PORT`, `TARGET_HOST`, `TARGET_PORT`.
    //! This is for development purposes. In production, these will be set by docker.
    
    // Temporary solution
    set_var("ROCKERY_HOST", "127.0.0.1");
    set_var("ROCKERY_PORT", "3000");

    set_var("TARGET_HOST", "127.0.0.1");
    set_var("TARGET_PORT", "5000");

    set_var("SPOOF_HOST_HEADER", "1");
}

pub fn resolve_to_socket_address(hostname: &str, port: &u16) -> SocketAddr{
    //! Resolves DNS hostname to IP address. Uses also port and 
    //! ultimately resolves provided data to `SocetAddr`.
    let resolved_dns_addresses = executor::block_on(
        net::lookup_host(format!("{}:{}", hostname, port))
    ).unwrap_or_else(|_| panic!("Provided DNS cannot be resolved!"));

    for addr in resolved_dns_addresses{
        return addr;
    };
    panic!("DNS resolution for {}:{} failed! Exiting...", hostname, port);
}

pub fn is_json_request(req: &Request<Body>) -> bool{
    //! Checks if request has application/json content type
    if !req.headers().contains_key("Content-Type"){
        return false;
    }

    if !(req.headers()["Content-Type"] == "application/json"){
        return false;
    }
    true
}

pub async fn parse_http_body_to_string(req: Request<Body>) -> Result<String, String>{
    //! Reads `Request<Body>` to bytes. These will be read as a `String`.
    let bytes = to_bytes(req.into_body()).await.map_err(|error|error.to_string())?;
    Ok(
        String::from_utf8(bytes.to_vec()).map_err(|e|e.to_string())?
    )
}

pub async fn parse_http_body_to_json(req: Request<Body>) -> Result<Value, String>{
    //! Parses `Request<Body>` to bytes. These will be read as a `String`
    //! and then parsed using `serde_json`, returning `Value`.
    let raw_request_body = parse_http_body_to_string(req).await?;
    let parsed_body: Value = serde_json::from_str(&raw_request_body).map_err(
        |_error| "HTTP request body must be in JSON format"
    )?;
    Ok(parsed_body)
}

pub fn json_message(message: &str) -> String{
    format!("{{\"msg\": \"{}\" }}\r\n", message)
}

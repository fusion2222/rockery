mod settings;
mod utils;
mod views;
mod db;
mod response;

use std::convert::Infallible;

use hyper::{Body, Request, Response, Server, Method};
use hyper::service::{make_service_fn, service_fn};

use db::{initialize_db, MockingRule};
use utils::set_env_vars;
use views::{RuleView};
use response::HTTPResponse;


async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    //! Handles every incoming Request and returns a Response.
    println!("[+] {} {}", req.method(), req.uri().to_string());

    // Make this more smarter - Allocating string for URLs can be done more efficiently.
    let processed_response : Result<Response<Body>, HTTPResponse> = match (req.method(), req.uri().path()) {
        (&Method::POST, "/rockery-mock/create-rule") => RuleView::create(req).await,
        (&Method::POST, "/rockery-mock/delete-rule") => RuleView::delete(req).await,
        _ => RuleView::default(req).await // Regular resend to target
    };
    match processed_response {
        Ok(resp) => Ok(resp),
        Err(error_response) => Ok(error_response.as_hyper_response())
    }
}


#[tokio::main]
async fn main() {
    set_env_vars();
    initialize_db().unwrap();

    println!(
        "[+] Gateway is listening on {}:{}",
        settings::ROCKERY_SOCKET_ADDRESS.ip(),
        settings::ROCKERY_SOCKET_ADDRESS.port()
    );

    println!(
        "[+] All requests will be redirected to {}:{}",
        settings::TARGET_SOCKET_ADDRESS.ip(),
        settings::TARGET_SOCKET_ADDRESS.port()
    );

    println!("[+] {} Rules exist", MockingRule::count_all().unwrap());

    let server = Server::bind(&settings::ROCKERY_SOCKET_ADDRESS).serve(
        make_service_fn(|_conn| async {
            return Ok::<_, Infallible>(service_fn(handle_request));
        })
    );

    if let Err(e) = server.await {
        panic!("Server error: {}", e);
    }
}

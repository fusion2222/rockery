use core::str::FromStr;
use std::convert::From;

use hyper::{ Body, Request, Response, Client, header::HeaderValue, Method };
use hyper::http::uri::{Scheme, Uri};
use hyper::http::StatusCode;
use serde_json::{ser, Value as JsonValue};

use crate::settings;
use crate::utils::{is_json_request, json_message, parse_http_body_to_json, parse_http_body_to_string};
use crate::response::HTTPResponse;
use crate::db::MockingRule;

/// View for handling mocking rules, which should
/// be called statically only. Initializing function
/// is `create`, `delete` and `default`.
pub struct RuleView {}
impl RuleView {
    fn get_json_request_url(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<String>{
        //! Extracts and validates request url. 
        let field_name = "_rockery_request_url".to_string();
        let mut output : Option<String> = None;

        match parsed_json.get(&field_name){
            Some(field) if !field.is_string() => error_messages.push(
                format!("{} must be a string", field_name)
            ),
            Some(field) => output = Some(field.as_str().unwrap_or_else(||"/").to_owned()),
            None => error_messages.push(format!("Field {} is required", field_name))
        };
        output
    }

    fn get_json_request_query(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<String>{
        let field_name = "_rockery_request_query".to_owned();
        let mut output : Option<String> = None;

        match parsed_json.get(&field_name){
            Some(field) if !field.is_string() => error_messages.push(
                format!("{} must be a string", field_name)
            ),
            Some(field) => output = match field.as_str() {
                Some(query) => Some(query.to_owned()),
                None => None
            },
            None => ()
        }
        output
    }

    fn get_json_request_method(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<Method>{
        let field_name = "_rockery_request_method".to_owned();
        let mut output : Option<Method> = None;

        match parsed_json.get(&field_name){
            Some(field) if field.is_string() => {
                let error_msg : String = format!(
                    "{} must be one of following: {}",
                    field_name,
                    settings::INTERCEPTABLE_METHODS
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                match Method::from_str(
                    &(field.as_str().unwrap_or("").to_uppercase())
                ) {
                    Ok(method) => {
                        if !settings::INTERCEPTABLE_METHODS.contains(&method){
                            error_messages.push(error_msg)
                        }else{
                            output = Some(method)
                        }
                    },
                    Err(_) => error_messages.push(error_msg),
                };
            },
            Some(_field) => error_messages.push(format!("{} must be a string", field_name)),
            None => error_messages.push(format!("Field {} is missing", field_name)),
        }
        output
    }

    fn get_json_request_data(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<String>{
        let field_name = "_rockery_request_data".to_owned();
        let mut output : Option<String> = None;
        
        match parsed_json.get(&field_name){
            Some(field) => {
                match ser::to_string(&field){
                    Ok(serialized_data) => output = Some(serialized_data),
                    Err(_) => error_messages.push(
                        format!(
                            "{} must be of JSON format in order to be serialized properly", field_name
                        )
                    )
                };
            },
            None => ()
        }
        output
    }

    fn get_json_response_status_code(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<i64>{
        let field_name = "_rockery_response_status_code".to_owned();
        let mut output : Option<i64> = None;
        
        match parsed_json.get(&field_name){
            Some(field) => {
                match field.as_i64() {
                    // Let's allow totally custom status codes for testing purposes...
                    Some(status_code) if StatusCode::from_u16(status_code as u16).is_err() => error_messages.push(
                        json_message(
                            &format!("Provided {} status code os not valid http status code", field_name)
                        )
                    ),
                    Some(status_code) => output = Some(status_code),
                    None => error_messages.push(
                        json_message(
                            &format!("{} must be an integer", field_name)
                        )
                    )
                }
            },
            None => error_messages.push(format!("Field {} is required", field_name)),
        }
        output
    }

    fn get_json_response_data(
        parsed_json: &JsonValue,
        error_messages: &mut Vec<String>
    ) -> Option<String> {
        let field_name = "_rockery_response_data".to_owned();
        let mut output : Option<String> = None;

        match parsed_json.get(&field_name){
            Some(field) => {
                match ser::to_string(&field){
                    Ok(serialized_data) => output = Some(serialized_data),
                    Err(_) => error_messages.push(
                        format!("{} must be of JSON format in order to be serialized properly", field_name)
                    )
                };
            },
            None => error_messages.push(format!("Field {} is required", field_name)),
        }
        output
    }

    fn create_mocking_rule_from_json(parsed_body: &JsonValue) -> Result<MockingRule, String>{
        //! Creates `MockingRule` from provided `serde_json:Value` or returns error message if validation fails.
        //! Just creates instance of `MockingRule` with filled attributes, but does not save to db!!! You need to
        //! call `MockingRule`'s `save` method in order to perform database save! 

        // let mut missing_fields : Vec<String> = vec![];
        let mut error_messages : Vec<String> = vec![];

        // Gather information about to-be-mocked request.
        let request_url : Option<String> = Self::get_json_request_url(parsed_body, &mut error_messages);
        let request_query : Option<String> = Self::get_json_request_query(parsed_body, &mut error_messages);
        let request_method : Option<Method> = Self::get_json_request_method(parsed_body, &mut error_messages);
        let request_data : Option<String> = Self::get_json_request_data(parsed_body, &mut error_messages);

        // Gather information about how to respond to to-be-mocked requests.
        let response_status_code : Option<i64> = Self::get_json_response_status_code(parsed_body, &mut error_messages);
        let response_data : Option<String> = Self::get_json_response_data(parsed_body, &mut error_messages);
        
        // Create mocking rule if possible.
        match (request_method, request_url, response_status_code, error_messages.len() > 0) {
            (Some(request_method_str ), Some(request_url_str), Some(response_status_code_int), false) => Ok(
                MockingRule {
                    id: None,
                    request_method: request_method_str,
                    request_url: request_url_str,
                    request_query: request_query,
                    request_data: request_data,
                    response_status_code: response_status_code_int,
                    response_data: response_data,
                }
            ),
            (_, _, _, true) => match serde_json::to_string(&error_messages) {
                Ok(error_message) => Err(error_message),
                Err(_) => Err("Fatal Error. Serialization of error messages failed!".to_owned()),
            },
            (_, _, _, false) => Err("Internal error. One of fields do not handle error messages properly".to_owned())
        }        
    }

    fn validate_rule_request(req: &Request<Body>) -> Result<(), HTTPResponse>{
        //! Validates if HTTP request fill needed general requirements - It is parseable, properly encoded, etc.
        if !is_json_request(&req){
            return Err(
                HTTPResponse{
                    body: json_message("Request Content-Type header must be application/json"),
                    status_code: StatusCode::BAD_REQUEST
                }
            );
        }
        Ok(())
    }

    pub async fn create(req: Request<Body>) -> Result<Response<Body>, HTTPResponse> {
        //! Hnadles requests, which attempt to create a new mocking rule.

        Self::validate_rule_request(&req)?;
        let parsed_body: JsonValue = parse_http_body_to_json(req).await.map_err(
            |error|
                HTTPResponse{
                    body: json_message(&error.to_string()),
                    status_code: StatusCode::UNPROCESSABLE_ENTITY
                }
        )?;
        
        let mut new_rule = Self::create_mocking_rule_from_json(&parsed_body).map_err(|error|
            HTTPResponse{
                body: json_message(&error),
                status_code: StatusCode::UNPROCESSABLE_ENTITY
            }
        )?;
        match new_rule.create(){
            Ok(_) => {
                return Ok(
                (HTTPResponse{
                    status_code: StatusCode::CREATED,
                    body: json_message(
                        &format!(
                            "Rule #{} for {} has been created successfully!",
                            new_rule.display_id(),
                            new_rule.request_url
                        )
                    ),
                }).as_hyper_response()
            )},
            Err(error_msg) => Err(
                HTTPResponse{
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    body: json_message(&error_msg),
                }
            )
        }
    }
    pub async fn delete(req: Request<Body>) -> Result<Response<Body>, HTTPResponse> {
        //! Hnadles requests, which attempt to delete existing mocking rule.
        Self::validate_rule_request(&req)?;
        let parsed_body: JsonValue = parse_http_body_to_json(req).await.map_err(
            |error|
                HTTPResponse{
                    status_code: StatusCode::UNPROCESSABLE_ENTITY,
                    body: json_message(&error)
                }
        )?;

        let mut error_messages : Vec<String> = vec![];

        let request_url : Option<String> = Self::get_json_request_url(&parsed_body, &mut error_messages);
        let request_query : Option<String> = Self::get_json_request_query(&parsed_body, &mut error_messages);
        let request_method : Option<Method> = Self::get_json_request_method(&parsed_body, &mut error_messages);
        let request_data : Option<String> = Self::get_json_request_data(&parsed_body, &mut error_messages);
        
        if error_messages.len() > 0{
            let (status_code, serialized_errors) = match serde_json::to_string(&error_messages){
                Ok(serialized_errors) => (StatusCode::UNPROCESSABLE_ENTITY, serialized_errors),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, json_message("Fatal Error: Failed to serialize errors")),
            };

            return Err(
                HTTPResponse{
                    status_code: status_code,
                    body: json_message(&serialized_errors)
                }
            );
        }
        
        let mut found_rules = MockingRule::find(
            &request_url,
            &request_query,
            &request_method,
            &request_data
        ).map_err(
            |e| HTTPResponse{
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                body: json_message(&e.to_string())
            }
        )?;

        if found_rules.len() == 0 {
            return Err(
                HTTPResponse{
                    status_code: StatusCode::NOT_FOUND,
                    body: json_message("Mocking rule does not exists and has not been deleted.")
                }
            )
        }
        match found_rules[0].delete() {
            Ok(_) => Ok(
                (HTTPResponse{
                    status_code: StatusCode::OK,
                    body: json_message("A rule has been deleted successfully")
                }).as_hyper_response()
            ),
            Err(error) => Err(
                HTTPResponse{
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    body: json_message(&error)
                }
            )
        }
    }

    pub async fn default(req: Request<Body>) -> Result<Response<Body>, HTTPResponse> {
        //! Hnadles requests, which will be possibly resent to target, waits
        //! for response, and returns the response.
        
        // TODO: Optimize!!!
        let req_uri = req.uri().clone();
        let method = req.method().clone();
        let headers = req.headers().clone();
        let http_version = req.version().clone();


        let request_body : String = parse_http_body_to_string(req).await.map_err(
            |error|
                HTTPResponse{
                    status_code: StatusCode::UNPROCESSABLE_ENTITY,
                    body: json_message(&error)
                }
        )?;

        if settings::INTERCEPTABLE_METHODS.contains(&method){
            let body = request_body.clone().trim().to_owned();

            match MockingRule::find(
                &Some(req_uri.to_string()),
                &req_uri.query().map(|o|o.to_owned()),
                &Some(method.clone()),
                &(if body != "" { Some(body) } else { None })
            ).map_err(
                |error|
                    HTTPResponse{
                        status_code: StatusCode::UNPROCESSABLE_ENTITY,
                        body: json_message(&error)
                    }
            )?.pop() {
                Some(rule) => {
                    println!("[+] Endpoint hit! Mocking response...");
                    let resp = Response::builder()
                        .status(
                            StatusCode::from_u16(
                                rule.response_status_code as u16
                            ).unwrap_or_else(
                                /*
                                    This happens if data are corrupted. Make sure
                                    we can store only valid status codes into DB.
                                */
                                |e| panic!(e.to_string())
                            )
                        )
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Allow-Headers", "*")
                        .header("Access-Control-Allow-Methods", "GET, PUT, POST, DELETE, HEAD, OPTIONS")
                        .header("Server", "Rockery - Rust Mocking Gateway")
                        .header("X-Mocked", "1")
                        .header("Content-Type", "application/json; charset=UTF-8")
                        .body(
                            Body::from(
                                rule.response_data.unwrap_or_else(||"-".to_owned())
                            )
                        ).map_err(|_error|
                            HTTPResponse{
                                status_code: StatusCode::UNPROCESSABLE_ENTITY,
                                body: json_message("[+] FATAL ERROR: Cannot generate mocked response")
                            }
                        )?;
                    return Ok(resp);
                },
                None => () 
            }
        }
        
        let client = Client::new();

        let target_uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(&*settings::TARGET_SOCKET_ADDRESS.to_string())
            .path_and_query(
                req_uri.clone().into_parts().path_and_query.unwrap()
            )
            .build()
            .unwrap();
        
        let mut proxy_request = Request::new(Body::from(request_body.clone()));
        *(proxy_request.uri_mut()) = target_uri;
        *(proxy_request.version_mut()) = http_version;
        *(proxy_request.method_mut()) = method;
        *(proxy_request.headers_mut()) = headers;

        if *settings::SPOOF_HOST_HEADER{
            proxy_request.headers_mut().insert(
                "Host", HeaderValue::from_static(&**settings::TARGET_HOST)
            );
        }
        
        Ok(
            client.request(proxy_request).await.map_err(
                |error|
                    HTTPResponse{
                        status_code: StatusCode::GATEWAY_TIMEOUT,
                        body: json_message(&error.to_string())
                    }
            )?
        )
    }
}

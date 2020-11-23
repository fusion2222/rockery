# Rockery - Rust Mocking Gateway

Simple gateway, which is able to mock `application/json` requests, in case target application is not available for some reason (Or even if it is).

## How to mock

Run gateway, and then send request to paths below. Rules are stored into in-memory non persistent sqlite3 DB.

- **POST** request to `/rockery-mock/create-rule` - For creating a mocking rule.
- **POST** request to `/rockery-mock/delete-rule` - For deleting existing mocking rule.

### Create rule example via curl

```bash
curl -X POST -H "Content-Type: application/json" -d '{"_rockery_request_url": "/url-to-be-mocked", "_rockery_request_method": "GET", "_rockery_response_status_code": 201, "_rockery_response_data": ""}' localhost:3000/rockery-mock/create-rule
```

### Delete rule example via curl

```bash
curl -X POST -H "Content-Type: application/json" -d '{"_rockery_request_url": "/url-to-be-mocked", "_rockery_request_method": "GET", "_rockery_response_status_code": 201, "_rockery_response_data": ""}' localhost:3000/rockery-mock/delete-rule
```

If any request will not match rules, it will be sent to target, and response will be returned.

## Network & HTTP issues

Keep in mind `HTTPS` connections cannot be intercepted. Even if you are intercepting `HTTP`, most of webservers are checking propper `Host` header. You can configure Rockery to spoof this header for you.

## Usage

Use at your own risk. Some connection types may not be handled properly - Websockets, or CORS requests or HTTP2, etc. Project is also not memory-optimized as it is far from being finished. A lot of work has to be done.

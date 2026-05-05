use crate::utilities::json_response_with_status;
use serde::Serialize;
use std::collections::HashMap;
use sysinfo::Pid;
use tiny_http::{Request, Response, StatusCode};
use url::form_urlencoded;

#[derive(Serialize)]
struct ProcessRunningResponse {
    process_is_running: bool,
}

pub fn handle_process_running_request(
    request: &Request,
    is_process_running_fn: impl Fn(Pid) -> bool,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url().to_string();
    let query_string = url.splitn(2, '?').nth(1).unwrap_or_default();
    let query: HashMap<String, String> = form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect();
    let pid: Pid = query
        .get("pid")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.into());

    let response_data = ProcessRunningResponse {
        process_is_running: is_process_running_fn(pid),
    };

    json_response_with_status(StatusCode(200), &response_data)
}

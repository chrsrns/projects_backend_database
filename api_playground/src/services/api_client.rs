use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Vec<ParameterInfo>,
    pub request_body: Option<RequestBodyInfo>,
    pub responses: HashMap<String, ResponseInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParameterInfo {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub schema_type: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RequestBodyInfo {
    pub description: String,
    pub required: bool,
    pub content_type: String,
    pub schema: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseInfo {
    pub description: String,
    pub content: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub async fn fetch_openapi_schema(base_url: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/api/openapi.json", base_url);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.ok() {
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read body: {}", e))?;
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse JSON: {}", e))
    } else {
        Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.status_text()
        ))
    }
}

pub async fn execute_request(
    method: &str,
    url: &str,
    token: Option<String>,
    body: Option<String>,
) -> Result<ApiResponse, String> {
    let request_builder = match method.to_uppercase().as_str() {
        "GET" => Request::get(url),
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        "PATCH" => Request::patch(url),
        _ => Request::get(url),
    };

    let request_builder = request_builder.header("Content-Type", "application/json");

    let request_builder = if let Some(t) = token {
        request_builder.header("Authorization", &format!("Bearer {}", t))
    } else {
        request_builder
    };

    let response = if let Some(b) = body {
        let request = request_builder
            .body(b)
            .map_err(|e| format!("Failed to build request: {}", e))?;
        request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
    } else {
        request_builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
    };

    let status = response.status();
    let status_text = response.status_text();

    let headers = response.headers().entries().map(|(k, v)| (k, v)).collect();

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;

    Ok(ApiResponse {
        status,
        status_text,
        headers,
        body,
    })
}

pub fn parse_endpoints_from_schema(schema: &serde_json::Value) -> Vec<EndpointInfo> {
    let mut endpoints = Vec::new();

    let components = schema
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_object())
        .cloned()
        .unwrap_or_default();

    if let Some(paths) = schema.get("paths").and_then(|p| p.as_object()) {
        for (path, path_item) in paths {
            for method in ["get", "post", "put", "delete", "patch"] {
                if let Some(operation) = path_item.get(method) {
                    let parameters = parse_parameters(operation, &components);
                    let request_body = parse_request_body(operation, &components);
                    let responses = parse_responses(operation, &components);

                    let tags = operation
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    endpoints.push(EndpointInfo {
                        path: path.clone(),
                        method: method.to_uppercase(),
                        summary: operation
                            .get("summary")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string(),
                        description: operation
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tags,
                        parameters,
                        request_body,
                        responses,
                    });
                }
            }
        }
    }

    endpoints.sort_by(|a, b| {
        let tag_cmp = a.tags.first().cmp(&b.tags.first());
        if tag_cmp == std::cmp::Ordering::Equal {
            a.path.cmp(&b.path)
        } else {
            tag_cmp
        }
    });

    endpoints
}

fn resolve_schema_ref(
    schema: &serde_json::Value,
    components: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    if let Some(ref_path) = schema.get("$ref").and_then(|r| r.as_str()) {
        // Parse "#/components/schemas/SchemaName"
        let parts: Vec<&str> = ref_path.split('/').collect();
        if parts.len() >= 4 && parts[1] == "components" && parts[2] == "schemas" {
            let schema_name = parts[3];
            if let Some(resolved) = components.get(schema_name) {
                return resolve_schema_ref(resolved, components);
            }
        }
    }
    schema.clone()
}

fn parse_parameters(
    operation: &serde_json::Value,
    _components: &serde_json::Map<String, serde_json::Value>,
) -> Vec<ParameterInfo> {
    operation
        .get("parameters")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|param| {
                    let name = param.get("name")?.as_str()?.to_string();
                    let location = param.get("in")?.as_str()?.to_string();
                    let required = param
                        .get("required")
                        .and_then(|r| r.as_bool())
                        .unwrap_or(false);
                    let schema_type = param
                        .get("schema")
                        .and_then(|s| s.get("type"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("string")
                        .to_string();
                    let description = param
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    Some(ParameterInfo {
                        name,
                        location,
                        required,
                        schema_type,
                        description,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_request_body(
    operation: &serde_json::Value,
    components: &serde_json::Map<String, serde_json::Value>,
) -> Option<RequestBodyInfo> {
    operation.get("requestBody").map(|rb| {
        let content = rb.get("content").and_then(|c| c.as_object());
        let (content_type, schema) = content
            .and_then(|map| {
                let first_key = map.keys().next()?.clone();
                let schema = map.get(&first_key)?.get("schema")?.clone();
                Some((first_key, schema))
            })
            .unwrap_or_else(|| ("application/json".to_string(), serde_json::Value::Null));

        let resolved_schema = resolve_schema_ref(&schema, components);

        RequestBodyInfo {
            description: rb
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
            required: rb
                .get("required")
                .and_then(|r| r.as_bool())
                .unwrap_or(false),
            content_type,
            schema: resolved_schema,
        }
    })
}

fn parse_responses(
    operation: &serde_json::Value,
    _components: &serde_json::Map<String, serde_json::Value>,
) -> HashMap<String, ResponseInfo> {
    let mut responses = HashMap::new();

    if let Some(resp_obj) = operation.get("responses").and_then(|r| r.as_object()) {
        for (code, resp) in resp_obj {
            responses.insert(
                code.clone(),
                ResponseInfo {
                    description: resp
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    content: resp.get("content").cloned(),
                },
            );
        }
    }

    responses
}

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

    let server_prefix = schema
        .get("servers")
        .and_then(|s| s.as_array())
        .and_then(|arr| arr.first())
        .and_then(|srv| srv.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

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

                    let full_path = if server_prefix.is_empty() || path.starts_with(&server_prefix)
                    {
                        path.clone()
                    } else {
                        format!("{}{}", server_prefix, path)
                    };

                    endpoints.push(EndpointInfo {
                        path: full_path,
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

pub(crate) fn resolve_schema_ref(
    schema: &serde_json::Value,
    components: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    resolve_schema_ref_inner(schema, components, &mut std::collections::HashSet::new())
}

/// Inner helper that carries a `visiting` set to break self-referential cycles
/// (e.g. `Schema` references itself).  When a cycle is detected the $ref is
/// left as-is so SchemaForm falls back gracefully rather than stack-overflowing.
fn resolve_schema_ref_inner(
    schema: &serde_json::Value,
    components: &serde_json::Map<String, serde_json::Value>,
    visiting: &mut std::collections::HashSet<String>,
) -> serde_json::Value {
    // Resolve $ref first, then recurse into the resolved value.
    if let Some(ref_path) = schema.get("$ref").and_then(|r| r.as_str()) {
        // Parse "#/components/schemas/SchemaName"
        let parts: Vec<&str> = ref_path.split('/').collect();
        if parts.len() >= 4 && parts[1] == "components" && parts[2] == "schemas" {
            let schema_name = parts[3].to_string();
            // Cycle guard: if we are already resolving this schema, return as-is.
            if visiting.contains(&schema_name) {
                return schema.clone();
            }
            if let Some(resolved) = components.get(&schema_name) {
                visiting.insert(schema_name.clone());
                let result = resolve_schema_ref_inner(resolved, components, visiting);
                visiting.remove(&schema_name);
                return result;
            }
        }
        return schema.clone();
    }

    // Deep-recurse into properties and items so nested $refs are resolved
    // before SchemaForm/ArrayField perform type dispatch (V42).
    let mut result = schema.clone();

    if let Some(obj) = result.as_object_mut() {
        // Resolve each property schema
        if let Some(properties) = obj.get("properties").cloned() {
            if let Some(props_map) = properties.as_object() {
                let resolved_props: serde_json::Map<String, serde_json::Value> = props_map
                    .iter()
                    .map(|(k, v)| (k.clone(), resolve_schema_ref_inner(v, components, visiting)))
                    .collect();
                obj.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(resolved_props),
                );
            }
        }

        // Resolve array items schema
        if let Some(items) = obj.get("items").cloned() {
            obj.insert(
                "items".to_string(),
                resolve_schema_ref_inner(&items, components, visiting),
            );
        }

        // Resolve anyOf / oneOf / allOf entries
        for keyword in ["anyOf", "oneOf", "allOf"] {
            if let Some(arr) = obj.get(keyword).cloned() {
                if let Some(variants) = arr.as_array() {
                    let resolved: Vec<serde_json::Value> = variants
                        .iter()
                        .map(|v| resolve_schema_ref_inner(v, components, visiting))
                        .collect();
                    obj.insert(keyword.to_string(), serde_json::Value::Array(resolved));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::resolve_schema_ref;
    use serde_json::json;

    /// V42: $ref inside array items must be resolved to concrete object schema
    /// so SchemaForm/ArrayField does not fall back to "string" type dispatch.
    #[test]
    fn test_v42_resolve_schema_ref_resolves_items_ref() {
        let mut components = serde_json::Map::new();
        components.insert(
            "Content".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "parts": { "type": "array", "items": { "type": "object" } }
                }
            }),
        );

        let schema = json!({
            "type": "object",
            "properties": {
                "contents": {
                    "type": "array",
                    "items": { "$ref": "#/components/schemas/Content" }
                }
            }
        });

        let resolved = resolve_schema_ref(&schema, &components);

        let items = resolved
            .get("properties")
            .and_then(|p| p.get("contents"))
            .and_then(|c| c.get("items"))
            .expect("items should exist");

        assert_eq!(
            items.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "items $ref must be resolved to concrete type before reaching SchemaForm (V42)"
        );
        assert!(
            items.get("$ref").is_none(),
            "$ref must not remain after resolution (V42)"
        );
    }

    /// Shallow $ref at top level must still resolve correctly (regression guard).
    #[test]
    fn test_v42_top_level_ref_still_resolves() {
        let mut components = serde_json::Map::new();
        components.insert(
            "Foo".to_string(),
            json!({ "type": "object", "properties": { "x": { "type": "string" } } }),
        );

        let schema = json!({ "$ref": "#/components/schemas/Foo" });
        let resolved = resolve_schema_ref(&schema, &components);

        assert_eq!(
            resolved.get("type").and_then(|t| t.as_str()),
            Some("object")
        );
    }

    /// V42 cycle guard: self-referential schema (e.g. Schema → Schema) must not
    /// cause a stack overflow; the recursive $ref is left as-is.
    #[test]
    fn test_v42_self_referential_schema_does_not_overflow() {
        let mut components = serde_json::Map::new();
        // "Schema" has a property "items" whose items are also "Schema" (direct cycle)
        components.insert(
            "Schema".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": { "$ref": "#/components/schemas/Schema" }
                    }
                }
            }),
        );

        let schema = json!({ "$ref": "#/components/schemas/Schema" });
        // Must complete without stack overflow; top-level type resolves correctly.
        let resolved = resolve_schema_ref(&schema, &components);
        assert_eq!(
            resolved.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "top-level Schema type must resolve despite cycle"
        );
    }
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

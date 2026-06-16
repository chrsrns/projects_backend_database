use crate::components::schema_form::{SchemaForm, default_value_for_schema};
use crate::services::api_client::{EndpointInfo, execute_request};
use leptos::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

#[component]
pub fn RequestBuilder(
    #[prop(into)] endpoint: Signal<Option<EndpointInfo>>,
    #[prop(into)] base_url: Signal<String>,
    #[prop(into)] token: RwSignal<String>,
    #[prop(into)] on_response: WriteSignal<Option<crate::services::api_client::ApiResponse>>,
) -> impl IntoView {
    let (path_params, set_path_params) = signal::<HashMap<String, String>>(HashMap::new());
    let (query_params, set_query_params) = signal::<HashMap<String, String>>(HashMap::new());
    let request_body: RwSignal<Value> = RwSignal::new(Value::Null);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (local_token, set_local_token) = signal(String::new());

    // Reset body signal when endpoint changes and pre-populate defaults
    Effect::new(move |_| {
        if let Some(ep) = endpoint.get() {
            if let Some(body_info) = ep.request_body {
                let default = default_value_for_schema(&body_info.schema);
                request_body.set(default);
            } else {
                request_body.set(Value::Null);
            }
            // Initialize local token from global when endpoint changes
            set_local_token.set(token.get());
        } else {
            request_body.set(Value::Null);
        }
    });

    // Sync global token changes back into the local input
    Effect::new(move |_| {
        let _ = endpoint.get(); // re-run on endpoint change too
        set_local_token.set(token.get());
    });

    let path_params_view = move || {
        let endpoint = endpoint.get()?;
        let params: Vec<_> = endpoint
            .parameters
            .iter()
            .filter(|p| p.location == "path")
            .cloned()
            .collect();

        if params.is_empty() {
            return None;
        }

        Some(view! {
            <div class="params-section">
                <h4>"Path Parameters"</h4>
                {params.into_iter().map(|param| {
                    let name = param.name.clone();
                    let name_label = param.name.clone();
                    let is_required = param.required;
                    let desc = param.description.clone();
                    view! {
                        <div class="param-field">
                            <label>
                                {name_label}
                                {move || if is_required { " *".to_string() } else { "".to_string() }}
                            </label>
                            <input
                                type="text"
                                placeholder=desc
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_path_params.update(|params| {
                                        params.insert(name.clone(), value);
                                    });
                                }
                            />
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        })
    };

    let query_params_view = move || {
        let endpoint = endpoint.get()?;
        let params: Vec<_> = endpoint
            .parameters
            .iter()
            .filter(|p| p.location == "query")
            .cloned()
            .collect();

        if params.is_empty() {
            return None;
        }

        Some(view! {
            <div class="params-section">
                <h4>"Query Parameters"</h4>
                {params.into_iter().map(|param| {
                    let name = param.name.clone();
                    let name_label = param.name.clone();
                    let is_required = param.required;
                    let desc = param.description.clone();
                    view! {
                        <div class="param-field">
                            <label>
                                {name_label}
                                {move || if is_required { " *".to_string() } else { "".to_string() }}
                            </label>
                            <input
                                type="text"
                                placeholder=desc
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_query_params.update(|params| {
                                        params.insert(name.clone(), value);
                                    });
                                }
                            />
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        })
    };

    let request_body_view = move || {
        let endpoint = endpoint.get()?;
        let body_info = endpoint.request_body?;
        let body_desc = body_info.description.clone();
        let schema = body_info.schema.clone();

        let json_preview = move || {
            let body = request_body.get();
            let json_str =
                serde_json::to_string_pretty(&body).unwrap_or_else(|_| "null".to_string());
            view! {
                <div class="json-preview-panel">
                    <label>"Payload Preview"</label>
                    <pre><code>{json_str}</code></pre>
                </div>
            }
        };

        Some(view! {
            <div class="params-section">
                <h4>"Request Body"</h4>
                <p class="body-description">{body_desc}</p>
                <div class="request-body-grid">
                    {json_preview}
                    <div>
                        <p class="payload-hint">"Edit payload values here"</p>
                        <SchemaForm
                            schema=schema
                            value=request_body
                        />
                    </div>
                </div>
            </div>
        })
    };

    let send_request = move |_| {
        let Some(endpoint) = endpoint.get() else {
            return;
        };

        set_loading.set(true);
        set_error.set(None);
        on_response.set(None);

        let mut url = format!("{}{}", base_url.get(), endpoint.path);

        // Replace path parameters
        for (key, value) in path_params.get().iter() {
            url = url.replace(&format!("{{{}}}", key), value);
        }

        // Add query parameters
        let query_string: Vec<String> = query_params
            .get()
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        if !query_string.is_empty() {
            url = format!("{}?{}", url, query_string.join("&"));
        }

        let method = endpoint.method.clone();
        let token_val = token.get();
        let token_opt = if token_val.is_empty() {
            None
        } else {
            Some(token_val)
        };

        let body_val = request_body.get();
        let body_opt =
            if body_val == Value::Null || body_val == Value::Object(serde_json::Map::new()) {
                None
            } else {
                serde_json::to_string(&body_val).ok()
            };

        leptos::task::spawn_local(async move {
            match execute_request(&method, &url, token_opt, body_opt).await {
                Ok(response) => {
                    on_response.set(Some(response));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    let selected_view = move || {
        let ep = endpoint.get();
        if let Some(endpoint) = ep {
            let method_badge_class =
                format!("method-badge method-{}", endpoint.method.to_lowercase());
            let method = endpoint.method.clone();
            let path = endpoint.path.clone();
            let description = endpoint.description.clone();

            view! {
                <div class="builder-header">
                    <h3>
                        <span class=method_badge_class>{method}</span>
                        {path}
                    </h3>
                    <p class="endpoint-description">{description}</p>
                </div>

                <div class="builder-form">
                    {path_params_view}
                    {query_params_view}
                    {request_body_view}

                    <div class="params-section">
                        <h4>"Authorization"</h4>
                        <div class="param-field" style="display: flex; gap: 0.5rem; align-items: center;">
                            <input
                                type="text"
                                placeholder="Enter your JWT token..."
                                prop:value=local_token
                                on:input=move |ev| {
                                    set_local_token.set(event_target_value(&ev));
                                }
                                style="flex: 1;"
                            />
                            <button
                                class="auth-global-btn"
                                on:click=move |_| {
                                    token.set(local_token.get());
                                }
                            >
                                "Set as token globally"
                            </button>
                        </div>
                    </div>

                    <div class="builder-actions" style="text-align: center; margin-top: 1.5rem;">
                        <button
                            class="send-request-btn"
                            on:click=send_request
                            disabled=loading
                        >
                            {move || if loading.get() { "Sending...".to_string() } else { "Send Request".to_string() }}
                        </button>
                    </div>

                    {move || {
                        if let Some(err) = error.get() {
                            Some(view! {
                                <div class="error-message">{err}</div>
                            })
                        } else {
                            None
                        }
                    }}
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="no-selection">
                    <p>"Select an endpoint from the sidebar to start building a request"</p>
                </div>
            }
            .into_any()
        }
    };

    view! {
        <div class="request-builder">
            {selected_view}
        </div>
    }
}

fn event_target_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

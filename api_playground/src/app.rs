use crate::components::endpoint_browser::EndpointBrowser;
use crate::components::request_builder::RequestBuilder;
use crate::components::response_viewer::ResponseViewer;
use crate::services::api_client::{
    ApiResponse, EndpointInfo, fetch_openapi_schema, parse_endpoints_from_schema,
};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn App() -> impl IntoView {
    let (base_url, set_base_url) = signal("http://localhost:8000".to_string());
    let token: RwSignal<String> = RwSignal::new(String::new());
    let (endpoints, set_endpoints) = signal(Vec::<EndpointInfo>::new());
    let (selected_endpoint, set_selected_endpoint) = signal::<Option<EndpointInfo>>(None);
    let (response, set_response) = signal::<Option<ApiResponse>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load endpoints on mount
    Effect::new(move |_| {
        let url = base_url.get();
        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match fetch_openapi_schema(&url).await {
                Ok(schema) => {
                    let parsed = parse_endpoints_from_schema(&schema);
                    set_endpoints.set(parsed);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    // Reset response when endpoint changes
    Effect::new(move |_| {
        let _ = selected_endpoint.get();
        set_response.set(None);
    });

    let refresh_endpoints = move |_| {
        let url = base_url.get();
        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match fetch_openapi_schema(&url).await {
                Ok(schema) => {
                    let parsed = parse_endpoints_from_schema(&schema);
                    set_endpoints.set(parsed);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="playground-container">
            <header class="playground-header">
                <div class="header-top">
                    <div class="header-title">
                        <h1>"API Playground"</h1>
                        <p class="subtitle">"Interactive API Explorer for Resume Profile Manager"</p>
                    </div>
                    <div class="token-status">
                        {move || {
                            let t = token.get();
                            if t.is_empty() {
                                view! {
                                    <div class="token-status-inner">
                                        <span class="token-badge token-none">"No Token"</span>
                                    </div>
                                }
                                .into_any()
                            } else {
                                let preview = if t.len() > 16 {
                                    format!("{}...", &t[..16])
                                } else {
                                    t.clone()
                                };
                                view! {
                                    <div class="token-status-inner">
                                        <span class="token-badge token-active">
                                            {"Token: "}
                                            {preview}
                                        </span>
                                        <button
                                            class="token-clear-btn"
                                            on:click=move |_| token.set(String::new())
                                        >
                                            "Clear"
                                        </button>
                                    </div>
                                }
                                .into_any()
                            }
                        }}
                    </div>
                </div>
                <div class="connection-config">
                    <label>"Base URL:"</label>
                    <input
                        type="text"
                        prop:value=base_url
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_base_url.set(value);
                        }
                    />
                    <button
                        class="refresh-btn"
                        on:click=refresh_endpoints
                    >
                        "Refresh Endpoints"
                    </button>
                </div>

                {move || {
                    if loading.get() {
                        Some(view! { <div class="loading-indicator">"Loading endpoints..."</div> })
                    } else {
                        None
                    }
                }}

                {move || {
                    if let Some(err) = error.get() {
                        Some(view! {
                            <div class="error-banner">
                                {"Failed to load endpoints: "}
                                {err}
                            </div>
                        })
                    } else {
                        None
                    }
                }}
            </header>

            <div class="playground-layout">
                <aside class="sidebar">
                    <EndpointBrowser
                        endpoints=endpoints
                        selected_endpoint_read=selected_endpoint
                        selected_endpoint=set_selected_endpoint
                    />
                </aside>

                <main class="main-content">
                    <div class="builder-panel">
                        <RequestBuilder
                            endpoint=selected_endpoint
                            base_url=base_url
                            token=token
                            on_response=set_response
                        />
                    </div>

                    <div class="response-panel">
                        <ResponseViewer response=response token=token />
                    </div>
                </main>
            </div>
        </div>
    }
}

fn event_target_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

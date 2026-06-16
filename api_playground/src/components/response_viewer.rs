use crate::services::api_client::ApiResponse;
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn ResponseViewer(
    #[prop(into)] response: Signal<Option<ApiResponse>>,
    #[prop(into)] token: RwSignal<String>,
) -> impl IntoView {
    let (show_token_banner, set_show_token_banner) = signal(false);
    let (detected_token, set_detected_token) = signal(String::new());
    let (active_tab, set_active_tab) = signal(0usize);

    // Watch for new responses and detect tokens
    Effect::new(move |_| {
        if let Some(resp) = response.get() {
            if resp.status >= 200 && resp.status < 300 {
                if let Some(t) = extract_token_from_json(&resp.body) {
                    // Only show if different from current token
                    if t != token.get() {
                        set_detected_token.set(t);
                        set_show_token_banner.set(true);
                    }
                }
            }
        }
    });

    // Reset to Body tab on new response
    Effect::new(move |_| {
        let _ = response.get();
        set_active_tab.set(0);
    });

    let use_token = move |_| {
        token.set(detected_token.get());
        set_show_token_banner.set(false);
    };

    let dismiss_token = move |_| {
        set_show_token_banner.set(false);
    };

    let status_class = move || {
        response.get().map(|resp| {
            if resp.status < 300 {
                "status-success".to_string()
            } else if resp.status < 400 {
                "status-redirect".to_string()
            } else if resp.status < 500 {
                "status-client-error".to_string()
            } else {
                "status-server-error".to_string()
            }
        })
    };

    let content_view = move || {
        if let Some(resp) = response.get() {
            let status = resp.status;
            let status_text = resp.status_text.clone();
            let body_str = resp.body.clone();
            let headers = resp.headers.clone();
            let status_class_str = status_class().unwrap_or_default();

            view! {
                <div class="response-header">
                    <h3>"Response"</h3>
                    <div class="status-line">
                        <span class=format!("status-code {}", status_class_str)>
                            {status.to_string()}
                        </span>
                        <span class="status-text">{status_text}</span>
                    </div>
                </div>

                {move || {
                    if show_token_banner.get() {
                        let t = detected_token.get();
                        let preview = if t.len() > 24 {
                            format!("{}...", &t[..24])
                        } else {
                            t.clone()
                        };
                        Some(view! {
                            <div class="token-banner">
                                <span class="token-banner-text">
                                    {"Token detected: "}
                                    <code>{preview}</code>
                                </span>
                                <div class="token-banner-actions">
                                    <button class="token-use-btn" on:click=use_token>
                                        "Use This Token"
                                    </button>
                                    <button class="token-dismiss-btn" on:click=dismiss_token>
                                        "Dismiss"
                                    </button>
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }
                }}

                <div class="response-tabs">
                    <button
                        class=move || format!("tab-btn {}", if active_tab.get() == 0 { "active" } else { "" })
                        on:click=move |_| set_active_tab.set(0)
                    >
                        "Body"
                    </button>
                    <button
                        class=move || format!("tab-btn {}", if active_tab.get() == 1 { "active" } else { "" })
                        on:click=move |_| set_active_tab.set(1)
                    >
                        "Headers"
                    </button>
                </div>

                <div class="response-content">
                    <div class=move || format!("tab-panel {}", if active_tab.get() == 0 { "active" } else { "" })>
                        <pre class="response-body">
                            <code>{format_json(&body_str)}</code>
                        </pre>
                    </div>

                    <div class=move || format!("tab-panel {}", if active_tab.get() == 1 { "active" } else { "" })>
                        <div class="headers-list">
                            {headers
                                .into_iter()
                                .map(|(key, value)| {
                                    view! {
                                        <div class="header-item">
                                            <span class="header-name">{key}</span>
                                            <span class="header-value">{value}</span>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="no-response">
                    <p>"Send a request to see the response here"</p>
                </div>
            }
            .into_any()
        }
    };

    view! {
        <div class="response-viewer">
            {content_view}
        </div>
    }
}

fn format_json(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .and_then(|v| serde_json::to_string_pretty(&v))
        .unwrap_or_else(|_| body.to_string())
}

/// Recursively search a JSON value for a "token" field containing a string.
fn extract_token_from_json(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    search_for_token(&value)
}

fn search_for_token(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            // Check direct "token" key first
            if let Some(token_val) = map.get("token") {
                if let Some(s) = token_val.as_str() {
                    return Some(s.to_string());
                }
            }
            // Otherwise search nested values
            for v in map.values() {
                if let Some(t) = search_for_token(v) {
                    return Some(t);
                }
            }
            None
        }
        Value::Array(arr) => {
            for v in arr {
                if let Some(t) = search_for_token(v) {
                    return Some(t);
                }
            }
            None
        }
        _ => None,
    }
}

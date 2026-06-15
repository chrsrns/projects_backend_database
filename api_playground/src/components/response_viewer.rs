use crate::services::api_client::ApiResponse;
use leptos::prelude::*;

#[component]
pub fn ResponseViewer(#[prop(into)] response: Signal<Option<ApiResponse>>) -> impl IntoView {
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

                <div class="response-tabs">
                    <button class="tab-btn active">"Body"</button>
                    <button class="tab-btn">"Headers"</button>
                </div>

                <div class="response-content">
                    <div class="tab-panel active">
                        <pre class="response-body">
                            <code>{format_json(&body_str)}</code>
                        </pre>
                    </div>

                    <div class="tab-panel">
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

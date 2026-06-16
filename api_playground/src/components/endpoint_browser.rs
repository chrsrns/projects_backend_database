use crate::services::api_client::EndpointInfo;
use leptos::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

#[component]
pub fn EndpointBrowser(
    #[prop(into)] endpoints: Signal<Vec<EndpointInfo>>,
    #[prop(into)] selected_endpoint_read: Signal<Option<EndpointInfo>>,
    #[prop(into)] selected_endpoint: WriteSignal<Option<EndpointInfo>>,
) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());
    let (expanded_tags, set_expanded_tags) = signal::<Vec<String>>(Vec::new());

    let grouped_endpoints = move || {
        let mut groups: HashMap<String, Vec<EndpointInfo>> = HashMap::new();
        let term = search_term.get().to_lowercase();

        for endpoint in endpoints.get() {
            if !term.is_empty()
                && !endpoint.path.to_lowercase().contains(&term)
                && !endpoint.summary.to_lowercase().contains(&term)
                && !endpoint.method.to_lowercase().contains(&term)
            {
                continue;
            }

            let tag = endpoint
                .tags
                .first()
                .cloned()
                .unwrap_or_else(|| "General".to_string());
            groups.entry(tag).or_default().push(endpoint);
        }

        let mut sorted_groups: Vec<(String, Vec<EndpointInfo>)> = groups.into_iter().collect();
        sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));
        sorted_groups
    };

    let toggle_tag = move |tag: String| {
        set_expanded_tags.update(|tags| {
            if tags.contains(&tag) {
                tags.retain(|t| t != &tag);
            } else {
                tags.push(tag);
            }
        });
    };

    view! {
        <div class="endpoint-browser">
            <div class="browser-header">
                <h2>"API Endpoints"</h2>
                <input
                    type="text"
                    placeholder="Search endpoints..."
                    class="search-input"
                    on:input=move |ev| {
                        set_search_term.set(event_target_value(&ev));
                    }
                />
            </div>

            <div class="endpoint-groups">
                {move || {
                    grouped_endpoints()
                        .into_iter()
                        .map(|(tag, endpoints)| {
                            let is_expanded = expanded_tags.get().contains(&tag);
                            let tag_clone = tag.clone();
                            let tag_clone2 = tag.clone();

                            view! {
                                <div class="endpoint-group">
                                    <button
                                        class="group-header"
                                        on:click=move |_| toggle_tag(tag_clone.clone())
                                    >
                                        <span class="group-title">{tag_clone2}</span>
                                        <span class="group-count">{endpoints.len()}</span>
                                        <span class="expand-icon">{move || if is_expanded { "▼".to_string() } else { "▶".to_string() }}</span>
                                    </button>

                                    {move || {
                                        if is_expanded {
                                            Some(view! {
                                                <div class="endpoints-list">
                                                    {endpoints
                                                        .clone()
                                                        .into_iter()
                                                        .map(|endpoint| {
                                                            let endpoint_clone = endpoint.clone();
                                                            let method = endpoint.method.clone();
                                                            let path = endpoint.path.clone();
                                                            let summary = endpoint.summary.clone();

                                                            let method_badge_class = format!("method-badge method-{}", method.to_lowercase());

                                                            let is_selected = selected_endpoint_read.get().as_ref()
                                                                .map(|s| s.path == endpoint.path && s.method == endpoint.method)
                                                                .unwrap_or(false);
                                                            let item_class = if is_selected {
                                                                "endpoint-item selected".to_string()
                                                            } else {
                                                                "endpoint-item".to_string()
                                                            };

                                                            view! {
                                                                <button
                                                                    class=item_class
                                                                    on:click=move |_| {
                                                                        selected_endpoint.set(Some(endpoint_clone.clone()));
                                                                    }
                                                                >
                                                                    <span class=method_badge_class>{method}</span>
                                                                    <span class="endpoint-path">{path}</span>
                                                                    <span class="endpoint-summary">{summary}</span>
                                                                </button>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()
                                                    }
                                                </div>
                                            })
                                        } else {
                                            None
                                        }
                                    }}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
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

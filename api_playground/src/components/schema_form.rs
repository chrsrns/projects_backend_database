use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;

/// Renders a dynamic form based on a JSON Schema, producing a JSON value.
#[component]
pub fn SchemaForm(
    #[prop(into)] schema: Value,
    #[prop(into)] value: RwSignal<Value>,
    #[prop(default = 0)] depth: usize,
) -> impl IntoView {
    // If the schema has no direct `properties` but has `oneOf`/`anyOf`, use the
    // first variant that has properties (e.g. `Part` is a union of object shapes).
    let effective_schema = if schema.get("properties").is_none() {
        schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .and_then(|arr| arr.as_array())
            .and_then(|variants| {
                variants
                    .iter()
                    .find(|v| v.get("properties").is_some())
                    .cloned()
            })
            .unwrap_or_else(|| schema.clone())
    } else {
        schema.clone()
    };

    let properties = effective_schema
        .get("properties")
        .and_then(|p| p.as_object())
        .cloned()
        .unwrap_or_default();

    let required: Vec<String> = effective_schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let required_set: std::collections::HashSet<String> = required.into_iter().collect();

    let field_entries: Vec<(String, Value, bool, usize)> = properties
        .into_iter()
        .map(|(name, schema_val)| {
            let is_required = required_set.contains(&name);
            (name, schema_val, is_required, depth)
        })
        .collect();

    if field_entries.is_empty() {
        let schema_json =
            serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "(invalid JSON)".to_string());
        return view! {
            <div class="schema-form-empty">
                <p>"No fields defined in schema"</p>
                <details class="schema-debug">
                    <summary>"Show raw schema"</summary>
                    <pre><code>{schema_json}</code></pre>
                </details>
            </div>
        }
        .into_any();
    }

    view! {
        <div class=format!("schema-form schema-form-depth-{}", depth)>
            {field_entries.into_iter().map(|(name, schema_val, is_required, field_depth)| {
                let field_name = name.clone();

                let field_value_signal = Signal::derive(move || {
                    value.get().get(&field_name).cloned().unwrap_or(Value::Null)
                });

                let value_for_update = value.clone();
                let name_for_update = name.clone();
                let on_change = Callback::new(move |new_val: Value| {
                    value_for_update.update(|v| {
                        if let Value::Object(map) = v {
                            map.insert(name_for_update.clone(), new_val);
                        } else {
                            let mut map = serde_json::Map::new();
                            map.insert(name_for_update.clone(), new_val);
                            *v = Value::Object(map);
                        }
                    });
                });

                view! {
                    <SchemaFieldInput
                        field_name=name.clone()
                        field_schema=schema_val
                        field_required=is_required
                        field_depth=field_depth
                        on_change=on_change
                        get_value=field_value_signal
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
    .into_any()
}

#[component]
fn SchemaFieldInput(
    #[prop(into)] field_name: String,
    #[prop(into)] field_schema: Value,
    field_required: bool,
    #[prop(default = 0)] field_depth: usize,
    on_change: Callback<Value>,
    #[prop(into)] get_value: Signal<Value>,
) -> impl IntoView {
    let schema_type = field_schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string")
        .to_string();

    let format_str = field_schema
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("")
        .to_string();

    let description = field_schema
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();

    let name_for_label = field_name.clone();
    let description_clone = description.clone();
    let label_view = move || {
        view! {
            <label class="schema-field-label">
                <span class="field-name">{name_for_label.clone()}</span>
                {if field_required {
                    view! { <span class="required-indicator">"*"</span> }.into_any()
                } else {
                    view! { <span class="optional-indicator">"(optional)"</span> }.into_any()
                }}
                {if !description_clone.is_empty() {
                    view! {
                        <span class="field-description">{description_clone.clone()}</span>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </label>
        }
    };

    match schema_type.as_str() {
        "string" => {
            if format_str == "uuid" {
                view! {
                    <div class="schema-field">
                        {label_view}
                        <input
                            type="text"
                            class="schema-input"
                            placeholder="550e8400-e29b-41d4-a716-446655440000"
                            prop:value=move || get_value.get().as_str().unwrap_or("").to_string()
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    on_change.run(Value::Null);
                                } else {
                                    on_change.run(Value::String(val));
                                }
                            }
                        />
                    </div>
                }
                .into_any()
            } else {
                let is_textarea = field_schema.get("maxLength").and_then(|v| v.as_u64()).map(|len| len > 100).unwrap_or(false);
                if is_textarea {
                    view! {
                        <div class="schema-field">
                            {label_view}
                            <textarea
                                class="schema-textarea"
                                rows="4"
                                prop:value=move || get_value.get().as_str().unwrap_or("").to_string()
                                on:input=move |ev| {
                                    let val = event_target_value(&ev);
                                    if val.is_empty() {
                                        on_change.run(Value::Null);
                                    } else {
                                        on_change.run(Value::String(val));
                                    }
                                }
                            />
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="schema-field">
                            {label_view}
                            <input
                                type="text"
                                class="schema-input"
                                prop:value=move || get_value.get().as_str().unwrap_or("").to_string()
                                on:input=move |ev| {
                                    let val = event_target_value(&ev);
                                    if val.is_empty() {
                                        on_change.run(Value::Null);
                                    } else {
                                        on_change.run(Value::String(val));
                                    }
                                }
                            />
                        </div>
                    }
                    .into_any()
                }
            }
        }
        "integer" | "number" => {
            view! {
                <div class="schema-field">
                    {label_view}
                    <input
                        type="number"
                        class="schema-input schema-number"
                        prop:value=move || get_value.get().as_f64().map(|v| v.to_string()).unwrap_or_default()
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            if val.is_empty() {
                                on_change.run(Value::Null);
                            } else if let Ok(n) = val.parse::<i64>() {
                                on_change.run(Value::Number(n.into()));
                            } else if let Ok(n) = val.parse::<f64>() {
                                if let Some(num) = serde_json::Number::from_f64(n) {
                                    on_change.run(Value::Number(num));
                                } else {
                                    on_change.run(Value::Null);
                                }
                            } else {
                                on_change.run(Value::Null);
                            }
                        }
                    />
                </div>
            }
            .into_any()
        }
        "boolean" => {
            let name_for_checkbox = field_name.clone();
            view! {
                <div class="schema-field schema-field-checkbox">
                    <label class="schema-checkbox-label">
                        <input
                            type="checkbox"
                            class="schema-checkbox"
                            prop:checked=move || get_value.get().as_bool().unwrap_or(false)
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                on_change.run(Value::Bool(checked));
                            }
                        />
                        <span class="field-name">{name_for_checkbox}</span>
                        {if field_required {
                            view! { <span class="required-indicator">"*"</span> }.into_any()
                        } else {
                            view! { <span class="optional-indicator">"(optional)"</span> }.into_any()
                        }}
                    </label>
                    {if !description.is_empty() {
                        view! {
                            <span class="field-description">{description.clone()}</span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            }
            .into_any()
        }
        "array" => {
            let item_schema = field_schema
                .get("items")
                .cloned()
                .unwrap_or(Value::Null);

            view! {
                <div class="schema-field">
                    {label_view}
                    <ArrayField
                        item_schema=item_schema
                        on_change=on_change
                        get_value=get_value
                    />
                </div>
            }
            .into_any()
        }
        "object" => {
            let child_signal = RwSignal::new(get_value.get());

            // Sync child signal back to parent, skipping the initial fire so
            // mounting this component does not trigger a spurious on_change
            // that cascades writes up the signal tree (V43).
            let parent_callback = on_change.clone();
            let mounted = StoredValue::new(false);
            Effect::new(move |_| {
                let val = child_signal.get();
                if mounted.get_value() {
                    parent_callback.run(val);
                } else {
                    mounted.set_value(true);
                }
            });

            view! {
                <div class=format!("schema-field schema-field-object schema-field-depth-{}", field_depth)>
                    {label_view}
                    <div class="schema-object-container">
                        <SchemaForm
                            schema=field_schema.clone()
                            value=child_signal
                            depth=field_depth + 1
                        />
                    </div>
                </div>
            }
            .into_any()
        }
        _ => {
            view! {
                <div class="schema-field">
                    {label_view}
                    <input
                        type="text"
                        class="schema-input"
                        placeholder=format!("Unknown type: {}", schema_type)
                        prop:value=move || get_value.get().to_string()
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            if val.is_empty() {
                                on_change.run(Value::Null);
                            } else {
                                on_change.run(Value::String(val));
                            }
                        }
                    />
                </div>
            }
            .into_any()
        }
    }
}

/// ArrayField owns a stable keyed list (`Vec<(u64, Value)>`) so that Leptos
/// `<For>` can diff by stable ID and never destroy/recreate item DOM nodes on
/// unrelated mutations (V44).  A single skip-first-run Effect serializes back
/// to `Value::Array` and propagates to the parent (V43).
#[component]
fn ArrayField(
    #[prop(into)] item_schema: Value,
    on_change: Callback<Value>,
    #[prop(into)] get_value: Signal<Value>,
) -> impl IntoView {
    // Convert the initial Value::Array into a stable keyed vec.
    let initial_items: Vec<(u64, Value)> = get_value
        .get()
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as u64, v))
        .collect();
    let next_id = StoredValue::new(initial_items.len() as u64);
    let local_items = RwSignal::new(initial_items);

    // Propagate local → parent as Value::Array, skipping the mount-time fire (V43).
    let mounted = StoredValue::new(false);
    Effect::new(move |_| {
        let arr = Value::Array(local_items.get().into_iter().map(|(_, v)| v).collect());
        if mounted.get_value() {
            on_change.run(arr);
        } else {
            mounted.set_value(true);
        }
    });

    let item_schema_clone = item_schema.clone();
    let add_item = move |_| {
        let default = default_value_for_schema(&item_schema_clone);
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        local_items.update(|items| items.push((id, default)));
    };

    let remove_item = move |id: u64| {
        local_items.update(|items| items.retain(|(k, _)| *k != id));
    };

    // Treat oneOf/anyOf-only schemas (e.g. untagged enums) as "object" for
    // dispatch purposes so they get StructuredArrayItem, not PrimitiveArrayItem.
    let item_type = {
        let t = item_schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if t.is_empty()
            && (item_schema.get("oneOf").is_some()
                || item_schema.get("anyOf").is_some()
                || item_schema.get("properties").is_some())
        {
            "object".to_string()
        } else if t.is_empty() {
            "string".to_string()
        } else {
            t
        }
    };

    view! {
        <div class="schema-array">
            <button
                class="array-add-btn"
                on:click=add_item
            >
                "+ Add Item"
            </button>

            <div class="array-items">
                <For
                    each=move || local_items.get()
                    key=|(id, _)| *id
                    children=move |(id, _item_val)| {
                        let item_type = item_type.clone();

                        // Item signal reads this item's value by stable ID (V44).
                        let item_signal = Signal::derive(move || {
                            local_items
                                .get()
                                .into_iter()
                                .find(|(k, _)| *k == id)
                                .map(|(_, v)| v)
                                .unwrap_or(Value::Null)
                        });

                        // Derive the display index reactively from position in list.
                        let display_idx = Signal::derive(move || {
                            local_items
                                .get()
                                .iter()
                                .position(|(k, _)| *k == id)
                                .unwrap_or(0)
                                + 1
                        });

                        // Item change writes into local_items by stable ID (V44).
                        let on_item_change = Callback::new(move |new_val: Value| {
                            local_items.update(|items| {
                                if let Some(entry) = items.iter_mut().find(|(k, _)| *k == id) {
                                    entry.1 = new_val;
                                }
                            });
                        });

                        view! {
                            <div class="array-item">
                                <div class="array-item-header">
                                    <span class="array-item-index">{move || format!("Item {}", display_idx.get())}</span>
                                    <button
                                        class="array-remove-btn"
                                        on:click=move |_| remove_item(id)
                                    >
                                        "Remove"
                                    </button>
                                </div>
                                <div class="array-item-content">
                                    {if item_type == "object" || item_type == "array" {
                                        view! {
                                            <StructuredArrayItem
                                                schema=item_schema.clone()
                                                get_value=item_signal
                                                on_change=on_item_change
                                            />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <PrimitiveArrayItem
                                                schema=item_schema.clone()
                                                on_change=on_item_change
                                                get_value=item_signal
                                            />
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

/// Wrapper component for object/array items inside an ArrayField.
///
/// Owns a stable `RwSignal` and a single `Effect` that syncs child edits back
/// to the parent array.  By being a component, the signal and effect are
/// created once at mount time — never inside the reactive `move ||` closure of
/// `ArrayField`, which would recreate them on every array mutation and cause an
/// infinite update loop (V43).
#[component]
fn StructuredArrayItem(
    #[prop(into)] schema: Value,
    #[prop(into)] get_value: Signal<Value>,
    on_change: Callback<Value>,
) -> impl IntoView {
    // Stable signal — created once when this component mounts.
    let child_signal = RwSignal::new(get_value.get());

    // Propagate child edits up to the parent array. Skip the first (mount-time)
    // fire so that adding an item does not immediately cascade writes up the
    // signal tree and cause an infinite loop (V43).
    let mounted = StoredValue::new(false);
    Effect::new(move |_| {
        let val = child_signal.get();
        if mounted.get_value() {
            on_change.run(val);
        } else {
            mounted.set_value(true);
        }
    });

    view! {
        <SchemaForm
            schema=schema
            value=child_signal
        />
    }
}

#[component]
fn PrimitiveArrayItem(
    #[prop(into)] schema: Value,
    on_change: Callback<Value>,
    #[prop(into)] get_value: Signal<Value>,
) -> impl IntoView {
    let schema_type = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string")
        .to_string();

    match schema_type.as_str() {
        "string" => {
            view! {
                <input
                    type="text"
                    class="schema-input"
                    prop:value=move || get_value.get().as_str().unwrap_or("").to_string()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        if val.is_empty() {
                            on_change.run(Value::Null);
                        } else {
                            on_change.run(Value::String(val));
                        }
                    }
                />
            }
            .into_any()
        }
        "integer" | "number" => {
            view! {
                <input
                    type="number"
                    class="schema-input schema-number"
                    prop:value=move || get_value.get().as_f64().map(|v| v.to_string()).unwrap_or_default()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        if val.is_empty() {
                            on_change.run(Value::Null);
                        } else if let Ok(n) = val.parse::<i64>() {
                            on_change.run(Value::Number(n.into()));
                        } else if let Ok(n) = val.parse::<f64>() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                on_change.run(Value::Number(num));
                            } else {
                                on_change.run(Value::Null);
                            }
                        } else {
                            on_change.run(Value::Null);
                        }
                    }
                />
            }
            .into_any()
        }
        "boolean" => {
            view! {
                <label class="schema-checkbox-label">
                    <input
                        type="checkbox"
                        class="schema-checkbox"
                        prop:checked=move || get_value.get().as_bool().unwrap_or(false)
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            on_change.run(Value::Bool(checked));
                        }
                    />
                    <span>"Value"</span>
                </label>
            }
            .into_any()
        }
        _ => {
            view! {
                <input
                    type="text"
                    class="schema-input"
                    prop:value=move || get_value.get().to_string()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        if val.is_empty() {
                            on_change.run(Value::Null);
                        } else {
                            on_change.run(Value::String(val));
                        }
                    }
                />
            }
            .into_any()
        }
    }
}

pub fn default_value_for_schema(schema: &Value) -> Value {
    // For oneOf/anyOf schemas (e.g. untagged enums like Part), use the first
    // variant that has properties to produce a meaningful default object.
    let effective = if schema.get("type").is_none() && schema.get("properties").is_none() {
        schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .and_then(|arr| arr.as_array())
            .and_then(|variants| {
                variants
                    .iter()
                    .find(|v| v.get("properties").is_some() || v.get("type").is_some())
                    .map(|v| std::borrow::Cow::Borrowed(v))
            })
            .unwrap_or_else(|| std::borrow::Cow::Borrowed(schema))
    } else {
        std::borrow::Cow::Borrowed(schema)
    };

    let schema_type = effective
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string");

    match schema_type {
        "string" => {
            if effective.get("format").and_then(|f| f.as_str()) == Some("uuid") {
                Value::String("550e8400-e29b-41d4-a716-446655440000".to_string())
            } else {
                Value::String("".to_string())
            }
        }
        "integer" | "number" => Value::Number(0.into()),
        "boolean" => Value::Bool(false),
        "array" => Value::Array(vec![]),
        "object" => {
            let mut map = serde_json::Map::new();
            if let Some(props) = effective.get("properties").and_then(|p| p.as_object()) {
                for (key, prop_schema) in props {
                    map.insert(key.clone(), default_value_for_schema(prop_schema));
                }
            }
            Value::Object(map)
        }
        _ => Value::Null,
    }
}

fn event_target_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn event_target_checked(ev: &web_sys::Event) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.checked())
        .unwrap_or(false)
}

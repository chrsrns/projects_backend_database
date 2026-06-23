use leptos::prelude::*;

const STORAGE_KEY: &str = "api_playground_heads_up_dismissed";

#[component]
pub fn HeadsUpDialog() -> impl IntoView {
    let (visible, set_visible) = signal(false);

    Effect::new(move |_| {
        let dismissed = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            .unwrap_or_default();

        if dismissed != "true" {
            set_visible.set(true);
        }
    });

    let dismiss = move || {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(STORAGE_KEY, "true");
        }
        set_visible.set(false);
    };

    let handle_keydown = move |e: leptos::ev::KeyboardEvent| {
        if e.key() == "Escape" {
            dismiss();
        }
    };

    let handle_backdrop_click = move |e: leptos::ev::MouseEvent| {
        // Only dismiss if clicking the overlay itself, not the dialog
        if e.target() == e.current_target() {
            dismiss();
        }
    };

    view! {
        {move || {
            if visible.get() {
                view! {
                    <div
                        class="heads-up-overlay"
                        on:click=handle_backdrop_click
                        on:keydown=handle_keydown
                        role="presentation"
                    >
                        <div
                            class="heads-up-dialog"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="heads-up-title"
                        >
                            <h2 id="heads-up-title" class="heads-up-title">"Heads up!"</h2>

                            <div class="heads-up-illustration">
                                <svg viewBox="0 0 400 200" xmlns="http://www.w3.org/2000/svg">
                                    /* Central backend server */
                                    <rect x="150" y="70" width="100" height="60" rx="8"
                                        fill="none" stroke="#38bdf8" stroke-width="2.5" />
                                    <rect x="160" y="82" width="80" height="6" rx="3"
                                        fill="#38bdf8" opacity="0.6" />
                                    <rect x="160" y="96" width="60" height="6" rx="3"
                                        fill="#38bdf8" opacity="0.4" />
                                    <rect x="160" y="110" width="70" height="6" rx="3"
                                        fill="#38bdf8" opacity="0.3" />

                                    /* Server label */
                                    <text x="200" y="155" text-anchor="middle"
                                        fill="#94a3b8" font-size="11" font-family="sans-serif">
                                        "Backend API"
                                    </text>

                                    /* Left connection line */
                                    <line x1="150" y1="100" x2="60" y2="100"
                                        stroke="#334155" stroke-width="2" stroke-dasharray="4 3" />
                                    <circle cx="60" cy="100" r="6" fill="#818cf8" />
                                    <text x="60" y="80" text-anchor="middle"
                                        fill="#94a3b8" font-size="10" font-family="sans-serif">
                                        "Project A"
                                    </text>

                                    /* Right top connection */
                                    <line x1="250" y1="85" x2="330" y2="55"
                                        stroke="#334155" stroke-width="2" stroke-dasharray="4 3" />
                                    <circle cx="330" cy="55" r="6" fill="#22c55e" />
                                    <text x="330" y="40" text-anchor="middle"
                                        fill="#94a3b8" font-size="10" font-family="sans-serif">
                                        "Project B"
                                    </text>

                                    /* Right bottom connection */
                                    <line x1="250" y1="115" x2="330" y2="145"
                                        stroke="#334155" stroke-width="2" stroke-dasharray="4 3" />
                                    <circle cx="330" cy="145" r="6" fill="#eab308" />
                                    <text x="330" y="170" text-anchor="middle"
                                        fill="#94a3b8" font-size="10" font-family="sans-serif">
                                        "Project C"
                                    </text>
                                </svg>
                            </div>

                            <p class="heads-up-body">
                                "This project, as well as this API Playground is more of a technical showcase. "
                                "This playground allows techies to check out my backend, but since the project IS a backend project, "
                                "it might not be user friendly nor intuitive."
                            </p>

                            <p class="heads-up-body">
                                "BUT — this project IS used by my other projects which are more presentable! Please check them out:"
                            </p>

                            <div class="heads-up-projects">
                                <a href="/resume_editor/resumes?source=backend" class="project-card">
                                    <div class="project-card-accent" />
                                    <h3>"Resume Editor"</h3>
                                    <p>"Create and edit resume data with full CRUD capabilities."</p>
                                </a>
                                <a href="https://chrsrns.github.io/css-resume/" target="_blank" rel="noopener noreferrer" class="project-card">
                                    <div class="project-card-accent accent-2" />
                                    <h3>"Resume Page"</h3>
                                    <p>"View my resume — a clean, readable presentation of my data."</p>
                                </a>
                            </div>

                            <button class="heads-up-ok-btn" on:click=move |_| dismiss()>
                                "Got it!"
                            </button>
                        </div>
                    </div>
                }
                .into_any()
            } else {
                ().into_any()
            }
        }}
    }
}

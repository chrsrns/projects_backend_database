// Live e2e test for POST /api/llm/gemini.
//
// Skipped (early return) when GEMINI_API_KEY is absent — never panics, never
// uses #[ignore]. See V39.
//
// Spawns a real Rocket server on port 19_001, drives it via reqwest over TCP.
// See V40 (≥ 150 ms startup sleep) and V41 (build_rocket_with_hub).

use api::realtime::Hub;
use shared::node_config::NodeConfig;

const PORT: u16 = 19_001;

#[test]
fn live_llm_endpoint() {
    // V39: skip when GEMINI_API_KEY absent.
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("[live_llm_endpoint] GEMINI_API_KEY not set — skipping live test");
            return;
        }
    };
    let _ = api_key; // key is read by build_rocket_with_hub from env; variable kept for clarity

    // V41: use build_rocket_with_hub so the real GeminiClient reads the key.
    let rocket = api::build_rocket_with_hub(Hub::new(), NodeConfig { port: 53421 }).configure(
        rocket::Config::figment()
            .merge(("address", "127.0.0.1"))
            .merge(("port", PORT)),
    );

    let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");

    runtime.block_on(async move {
        let ignited = rocket.ignite().await.expect("ignite rocket");
        let shutdown = ignited.shutdown();
        let server = rocket::tokio::spawn(async move {
            let _ = ignited.launch().await;
        });

        // V40: sleep ≥ 150 ms before first request.
        rocket::tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let client = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}/api/llm/gemini", PORT);

        // --- Assertion 1: valid request → 200 + V7 body shape ---
        let valid_payload = serde_json::json!({
            "contents": [{"parts": [{"text": "Say the word 'hello' only."}]}]
        });
        let resp = client
            .post(&base)
            .header("Content-Type", "application/json")
            .body(valid_payload.to_string())
            .send()
            .await
            .expect("reqwest send");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "expected 200 for valid LLM request"
        );

        // V7: body wrapped in Response<T> — must have "body" key.
        let text = resp.text().await.expect("response body text");
        let json: serde_json::Value =
            serde_json::from_str(&text).expect("response body is valid JSON");
        assert!(
            json.get("body").is_some(),
            "response missing 'body' key (V7): {json}"
        );

        // --- Assertion 2: "model" field present → 400 (V25 live) ---
        let model_payload = serde_json::json!({
            "model": "gemini-1.5-flash",
            "contents": [{"parts": [{"text": "hello"}]}]
        });
        let resp2 = client
            .post(&base)
            .header("Content-Type", "application/json")
            .body(model_payload.to_string())
            .send()
            .await
            .expect("reqwest send");

        assert_eq!(
            resp2.status().as_u16(),
            400,
            "expected 400 when 'model' field present (V25)"
        );

        // V7: error response also wrapped in Response<T>.
        let text2 = resp2.text().await.expect("error response body text");
        let json2: serde_json::Value =
            serde_json::from_str(&text2).expect("error response body is valid JSON");
        assert!(
            json2.get("body").is_some(),
            "400 response missing 'body' key (V7): {json2}"
        );

        shutdown.notify();
        let _ = server.await;
    });
}

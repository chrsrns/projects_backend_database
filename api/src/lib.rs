#[macro_use]
extern crate rocket;

use shared::node_config::NodeConfig;
use utoipa::OpenApi;

pub mod auth;
pub mod openapi;
pub mod realtime;
pub mod route_handlers;
pub mod ws_handler;

use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Once;
use std::time::SystemTime;

use rocket::Request;
use rocket::http::Status;
use rocket::http::uri::{Segments, fmt::Path as UriPath};
use rocket::request::FromSegments;
use rocket::response::{Responder, Response};
use route_handlers::resume::*;

static LOG_INIT: Once = Once::new();

fn parse_level_filter(raw: &str) -> log::LevelFilter {
    match raw.trim().to_ascii_lowercase().as_str() {
        "off" => log::LevelFilter::Off,
        "error" => log::LevelFilter::Error,
        "warn" | "warning" | "critical" => log::LevelFilter::Warn,
        "info" | "normal" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    }
}

fn log_level_from_env() -> log::LevelFilter {
    if let Ok(v) = std::env::var("RUST_LOG") {
        return parse_level_filter(&v);
    }

    if let Ok(v) = std::env::var("ROCKET_LOG_LEVEL") {
        return match v.to_ascii_lowercase().as_str() {
            "off" => log::LevelFilter::Off,
            "critical" => log::LevelFilter::Warn,
            "normal" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            _ => log::LevelFilter::Info,
        };
    }

    log::LevelFilter::Info
}

pub fn init_logging() {
    LOG_INIT.call_once(|| {
        let level = log_level_from_env();

        let exe_dir: PathBuf = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let timestamp = humantime::format_rfc3339_seconds(SystemTime::now());
        let log_path = exe_dir.join(format!("server-{}.log", timestamp));
        println!("Log path: {}", log_path.display());

        let stdout_dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    message
                ))
            })
            .chain(std::io::stdout());

        let mut dispatch = fern::Dispatch::new().level(level).chain(stdout_dispatch);

        match fern::log_file(&log_path) {
            Ok(file) => {
                let file_dispatch = fern::Dispatch::new()
                    .format(|out, message, record| {
                        let msg = message.to_string();
                        let stripped = strip_ansi_escapes::strip(&msg);
                        out.finish(format_args!(
                            "[{} {}] {}",
                            humantime::format_rfc3339_seconds(SystemTime::now()),
                            record.level(),
                            String::from_utf8_lossy(&stripped)
                        ))
                    })
                    .chain(file);

                dispatch = dispatch.chain(file_dispatch);
            }
            Err(err) => {
                eprintln!(
                    "Failed to create log file at {}: {}",
                    log_path.display(),
                    err
                );
            }
        };

        let _ = dispatch.apply();
    });
}

pub fn build_rocket(node_cfg: NodeConfig) -> rocket::Rocket<rocket::Build> {
    build_rocket_with_hub(realtime::Hub::new(), node_cfg)
}

struct ProxyResponse {
    status: Status,
    content_type: Option<String>,
    body: Vec<u8>,
}

struct FrontendProxyPath(String);

impl<'r> FromSegments<'r> for FrontendProxyPath {
    type Error = std::convert::Infallible;

    fn from_segments(segments: Segments<'r, UriPath>) -> Result<Self, Self::Error> {
        let segments_vec: Vec<_> = segments.collect();

        Ok(Self(segments_vec.join("/")))
    }
}

impl<'r> Responder<'r, 'static> for ProxyResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut builder = Response::build();
        builder.status(self.status);
        if let Some(content_type) = self.content_type {
            builder.raw_header("Content-Type", content_type);
        }

        builder
            .sized_body(self.body.len(), Cursor::new(self.body))
            .ok()
    }
}

async fn proxy_frontend_path(path: &str, node_port: u16) -> Result<ProxyResponse, Status> {
    let upstream_url = format!("http://localhost:{}{path}", node_port);
    log::info!(
        "frontend proxy step=prepare_request path={} upstream_url={}",
        path,
        upstream_url
    );

    let upstream_response = reqwest::get(&upstream_url).await.map_err(|err| {
        log::error!(
            "frontend proxy step=send_request path={} upstream_url={} error={}",
            path,
            upstream_url,
            err
        );
        Status::BadGateway
    })?;

    log::info!(
        "frontend proxy step=received_response path={} upstream_url={} upstream_status={}",
        path,
        upstream_url,
        upstream_response.status()
    );

    let status =
        Status::from_code(upstream_response.status().as_u16()).unwrap_or(Status::BadGateway);
    log::info!(
        "frontend proxy step=map_status path={} upstream_status={} rocket_status={}",
        path,
        upstream_response.status(),
        status
    );

    let content_type = upstream_response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    log::info!(
        "frontend proxy step=read_headers path={} content_type={}",
        path,
        content_type.as_deref().unwrap_or("<none>")
    );

    let body = upstream_response.bytes().await.map_err(|err| {
        log::error!(
            "frontend proxy step=read_body path={} upstream_url={} error={}",
            path,
            upstream_url,
            err
        );
        Status::BadGateway
    })?;

    log::info!(
        "frontend proxy step=complete path={} status={} body_bytes={}",
        path,
        status,
        body.len()
    );

    Ok(ProxyResponse {
        status,
        content_type,
        body: body.to_vec(),
    })
}

#[get("/", rank = 100)]
async fn frontend_index_proxy_handler(
    node_cfg: &rocket::State<NodeConfig>,
) -> Result<ProxyResponse, Status> {
    proxy_frontend_path("/", node_cfg.port).await
}

#[get("/resume_editor/<path..>?<query..>", rank = 101)]
async fn frontend_proxy_handler(
    path: FrontendProxyPath,
    query: Option<std::collections::HashMap<String, String>>,
    node_cfg: &rocket::State<NodeConfig>,
) -> Result<ProxyResponse, Status> {
    let normalized_path = path.0;

    let full_path = match query {
        Some(q) if !q.is_empty() => {
            let query_string: String = q
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("/resume_editor/{}?{}", normalized_path, query_string)
        }
        _ => format!("/resume_editor/{}", normalized_path),
    };

    proxy_frontend_path(&full_path, node_cfg.port).await
}

pub fn build_rocket_with_hub(
    hub: realtime::Hub,
    node_cfg: NodeConfig,
) -> rocket::Rocket<rocket::Build> {
    let allowed_origins = rocket_cors::AllowedOrigins::all();

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![
            rocket::http::Method::Get,
            rocket::http::Method::Post,
            rocket::http::Method::Put,
            rocket::http::Method::Delete,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .manage(hub)
        .manage(node_cfg)
        .mount(
            "/api",
            routes![
                auth_handler::register_handler,
                auth_handler::login_handler,
                auth_handler::me_handler,
                auth_handler::logout_handler,
                ws_handler::ws_handler,
                resume_handler::list_resumes_handler,
                resume_handler::list_resume_handler,
                resume_handler::create_resume_handler,
                resume_handler::update_resume_handler,
                resume_handler::delete_resume_handler,
                skills_handler::list_skills_handler,
                skills_handler::create_skill_handler,
                skills_handler::update_skill_handler,
                skills_handler::delete_skill_handler,
                languages_handler::list_languages_handler,
                languages_handler::create_language_handler,
                languages_handler::update_language_handler,
                languages_handler::delete_language_handler,
                frameworks_handler::list_frameworks_handler,
                frameworks_handler::create_framework_handler,
                frameworks_handler::update_framework_handler,
                frameworks_handler::delete_framework_handler,
                education_handler::list_educations_handler,
                education_handler::create_education_handler,
                education_handler::update_education_handler,
                education_handler::delete_education_handler,
                education_handler::list_education_key_points_handler,
                education_handler::create_education_key_point_handler,
                education_handler::update_education_key_point_handler,
                education_handler::delete_education_key_point_handler,
                work_experiences_handler::list_work_experiences_handler,
                work_experiences_handler::create_work_experience_handler,
                work_experiences_handler::update_work_experience_handler,
                work_experiences_handler::delete_work_experience_handler,
                work_experiences_handler::list_work_experience_key_points_handler,
                work_experiences_handler::create_work_experience_key_point_handler,
                work_experiences_handler::update_work_experience_key_point_handler,
                work_experiences_handler::delete_work_experience_key_point_handler,
                portfolio_projects_handler::list_portfolio_projects_handler,
                portfolio_projects_handler::create_portfolio_project_handler,
                portfolio_projects_handler::update_portfolio_project_handler,
                portfolio_projects_handler::delete_portfolio_project_handler,
                portfolio_projects_handler::list_portfolio_key_points_handler,
                portfolio_projects_handler::create_portfolio_key_point_handler,
                portfolio_projects_handler::update_portfolio_key_point_handler,
                portfolio_projects_handler::delete_portfolio_key_point_handler,
                portfolio_projects_handler::list_portfolio_technologies_handler,
                portfolio_projects_handler::create_portfolio_technology_handler,
                portfolio_projects_handler::update_portfolio_technology_handler,
                portfolio_projects_handler::delete_portfolio_technology_handler,
            ],
        )
        .mount(
            "/resume_builder",
            routes![frontend_index_proxy_handler, frontend_proxy_handler],
        )
        .mount(
            "/",
            utoipa_swagger_ui::SwaggerUi::new("/api/docs/<_..>")
                .url("/api/openapi.json", openapi::ApiDoc::openapi()),
        )
}

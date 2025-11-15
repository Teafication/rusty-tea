mod config;
mod handlers;
mod middleware;
mod models;
mod services;

use axum::{
    extract::DefaultBodyLimit,
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

use config::Config;
use middleware::check_api_key;
use services::{VoskService, DatabaseService, RagService, LlmService, ElevenLabsService, VoiceSessionService};

#[derive(Clone)]
pub struct AppState {
    name: String,
    version: String,
    vosk_service: VoskService,
    database_service: Arc<DatabaseService>,
    rag_service: Option<Arc<RagService>>,
    llm_service: Arc<LlmService>,
    elevenlabs_service: Arc<ElevenLabsService>,
    voice_sessions: VoiceSessionService,
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")))
        .init();

    info!("Starting Rusty Tea server...");
    info!("Config: {:?}", config);

    // Initialize database service
    let database_service = match DatabaseService::new(&config.database_url).await {
        Ok(db) => {
            info!("Database service initialized");
            Arc::new(db)
        }
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            panic!("Database initialization failed: {}", e);
        }
    };

    // Initialize Qdrant RAG service (optional for Phase 1 testing)
    let rag_service = match RagService::new(&config.qdrant_url).await {
        Ok(rag) => {
            info!("Qdrant RAG service initialized");
            Some(Arc::new(rag))
        }
        Err(e) => {
            tracing::warn!("Qdrant initialization failed (Phase 2 feature): {}", e);
            None
        }
    };

    // Initialize LLM service
    let llm_service = match LlmService::new(
        &config.openrouter_api_key,
        &config.openrouter_base_url,
        &config.openrouter_chat_model_lite,
    ) {
        Ok(llm) => {
            info!("LLM service initialized");
            Arc::new(llm)
        }
        Err(e) => {
            tracing::error!("Failed to initialize LLM service: {}", e);
            panic!("LLM service initialization failed: {}", e);
        }
    };

    // Initialize ElevenLabs TTS service
    let elevenlabs_service = match ElevenLabsService::new(
        config.elevenlabs_api_key.clone(),
        config.elevenlabs_voice_id.clone(),
    ) {
        Ok(tts) => {
            info!("ElevenLabs TTS service initialized");
            Arc::new(tts)
        }
        Err(e) => {
            tracing::error!("Failed to initialize ElevenLabs service: {}", e);
            panic!("ElevenLabs service initialization failed: {}", e);
        }
    };

    // Initialize voice session service (in-memory, ephemeral)
    let voice_sessions = VoiceSessionService::new(30); // 30 minute TTL
    voice_sessions.clone().start_cleanup_task();
    info!("Voice session service initialized with 30-minute TTL");

    let state = AppState {
        name: "Rusty Tea".to_string(),
        version: "0.1.0".to_string(),
        vosk_service: VoskService::new(config.vosk_model_path.clone()),
        database_service,
        rag_service,
        llm_service,
        elevenlabs_service,
        voice_sessions,
    };

    let app = Router::new()
        // Health endpoints (public, no auth required)
        .route("/health", get(handlers::health_check))
        .route("/status", get(handlers::server_status))
        // Protected endpoints (require API key)
        .route(
            "/api/v1/transcriptions",
            post(handlers::transcribe_batch).layer(DefaultBodyLimit::max(100 * 1024 * 1024)), // 100MB limit
        )
        .route("/api/v1/transcribe/stream", get(handlers::transcribe_stream))
        .route(
            "/voice-chat",
            post(handlers::voice_chat).layer(DefaultBodyLimit::max(10 * 1024 * 1024)), // 10MB limit for voice
        )
        .with_state(Arc::new(state))
        .layer(from_fn(check_api_key))
        .layer(TraceLayer::new_for_http());

    let address = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect(&format!("Failed to bind to {}", address));

    info!("Server running on http://{}", address);
    info!("Endpoints:");
    info!("  GET  /health");
    info!("  GET  /status");
    info!("  POST /api/v1/transcriptions (batch)");
    info!("  WS   /api/v1/transcribe/stream (streaming)");
    info!("  POST /voice-chat (voice conversation)");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

# Rusty Tea - Quick Dev Guide

## 🚀 Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run with Docker (recommended)
docker-compose -f docker/docker-compose.yml up -d

# OR run locally with Postgres + Qdrant running
cargo run
```

## 📦 Architecture

**Stack:** Rust + Axum web framework

**3 Services:**

- **API** (Rust/Axum) port 8765 - Voice chat, transcription, TTS
- **PostgreSQL** port 5433 - Conversations, messages (internal only)
- **Qdrant** port 6334 - Vector embeddings (internal only)

**AppState Services:**

```rust
state.database_service.pool()       // PostgreSQL connection pool (sqlx)
state.rag_service.client()           // Qdrant vector client
state.llm_service.client()           // OpenRouter LLM (async-openai)
state.vosk_service                   // Vosk speech recognition (local)
state.elevenlabs_service             // ElevenLabs TTS
state.voice_sessions                 // Ephemeral session storage (30min TTL)
```

## 🗄️ Database

**Auto-migrations:**

- Migrations in `migrations/` folder auto-run on app startup
- Uses sqlx `migrate!()` macro - no manual SQL needed
- Creates 5 tables: conversations, messages, documents, embeddings, api_usage
- All indexes and constraints auto-applied

**Schema:**

```
conversations: id, user_id, title, created_at, updated_at
messages: id, conversation_id, role, content, tokens_used, created_at
documents: id, conversation_id, file_name, content, indexed, created_at
embeddings: id, document_id, chunk_text, vector_id, created_at
api_usage: id, conversation_id, model, prompt_tokens, completion_tokens, cost, created_at
```

## 🔑 Environment Variables

Set in `.env`:

```bash
# Auth
API_KEY=your_api_key_here

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000  # Internal container port
RUST_LOG=info

# Database (internal Docker network)
DATABASE_URL=postgresql://app@postgres:5432/rusty_tea_db

# Vector DB (internal)
QDRANT_URL=http://qdrant:6333

# LLM (OpenRouter)
OPENROUTER_API_KEY=sk-or-v1-your-key
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_CHAT_MODEL_LITE=meta-llama/llama-3.1-8b-instruct

# TTS (ElevenLabs)
ELEVENLABS_API_KEY=sk_your_key
ELEVENLABS_VOICE_ID=your_voice_id

# Vosk Model
VOSK_MODEL_PATH=/models/vosk-model-small-en-us-0.15
```

**Ports (host → container):**

- API: 8765 → 3000
- PostgreSQL: 5433 → 5432 (internal only)
- Qdrant: 6334 → 6333 (internal only)

## 🧪 Testing

```bash
# All tests (includes container communication tests)
cargo test

# Specific module
cargo test config::tests
cargo test container_integration_tests

# With output
cargo test -- --nocapture

# Release (like Docker build)
cargo test --release
```

## 🔐 API Authentication

All endpoints except `/health` and `/status` require Bearer token:

```bash
curl -H "Authorization: Bearer your_token" \
  http://localhost:8765/voice-chat
```

## 📡 Current Endpoints

```
GET  /health                          # Server health
GET  /status                          # Server status + endpoints
POST /api/v1/transcriptions           # Batch transcription (16kHz WAV)
WS   /api/v1/transcribe/stream        # Streaming transcription
POST /voice-chat                      # Voice chat (WAV → MP3, requires Bearer token)
```

**Voice Chat:**

- Input: multipart/form-data with `audio` (16kHz mono WAV) + `voice_session_id` (UUID)
- Output: audio/mpeg (MP3)
- Session: 30min TTL, in-memory only (privacy-friendly)

## 🔄 Docker Compose

```bash
# Start
docker-compose -f docker/docker-compose.yml up -d

# Logs
docker-compose -f docker/docker-compose.yml logs -f api

# Health checks
curl http://localhost:8765/health      # API
curl http://localhost:6334/health      # Qdrant (internal)
docker exec rusty_tea_postgres pg_isready -U postgres  # PostgreSQL

# Stop
docker-compose -f docker/docker-compose.yml down

# Rebuild
docker-compose -f docker/docker-compose.yml up --build -d
```

## 📝 Code Structure

```
src/
├── main.rs              # Entry, routing, service initialization
├── config.rs            # Environment variables loader
├── models.rs            # DTOs and response types
├── middleware.rs        # Bearer token auth
├── handlers/            # HTTP endpoints
│   ├── health.rs
│   ├── transcription.rs
│   └── voice_chat.rs     # Voice chat with TTS
└── services/            # Business logic
    ├── vosk_service.rs          # Vosk speech-to-text (local)
    ├── database_service.rs      # PostgreSQL pooling
    ├── qdrant_service.rs        # Vector DB client
    ├── llm_service.rs           # OpenRouter LLM + Tea personality
    ├── elevenlabs_service.rs    # ElevenLabs TTS
    └── voice_session_service.rs # Ephemeral sessions (30min TTL)

migrations/
└── 20240101000001_init_schema.sql    # Auto-runs on startup

tests/
├── integration_test.rs       # Cross-module tests
└── container_tests.rs        # Container communication tests
```

## 🚢 Development Workflow

1. **Make code changes**
2. **Test locally:** `cargo test --release`
3. **Build Docker:** `docker-compose up --build -d`
4. **Verify:** `curl http://localhost:3000/health`

## 🛠️ Common Tasks

**Add new endpoint:**

```rust
// 1. Create handler in handlers/
pub async fn my_endpoint(State(state): State<Arc<AppState>>) -> Result<Json<Response>> {
    // Use state.database_service, state.rag_service, state.llm_service
}

// 2. Add route in main.rs
.route("/api/v1/my-endpoint", get(handlers::my_endpoint))
```

**Query database:**

```rust
let result = sqlx::query_as::<_, MyType>(
    "SELECT * FROM my_table WHERE id = $1"
)
    .bind(my_id)
    .fetch_one(state.database_service.pool())
    .await?;
```

**Call LLM:**

```rust
let response = state.llm_service.client().chat().create(request).await?;
```

**Search Qdrant:**

```rust
let results = state.rag_service.client()
    .search_points("documents", query_vector, 4, None)
    .await?;
```

## 🚨 Troubleshooting

**"Failed to connect to database"**

- Ensure PostgreSQL container is running: `docker ps`
- Check connection string in `.env`
- Verify port 5432 is open

**"Qdrant health check failed"**

- Check Qdrant container: `docker ps`
- Verify port 6333 open: `curl http://localhost:6333/health`

**"Migrations failed"**

- Check migrations folder exists: `ls migrations/`
- Verify SQL syntax is correct
- Check PostgreSQL has correct permissions

**"API key rejected"**

- Verify `Authorization: Bearer <token>` header matches `BEARER_TOKEN` in `.env`
- Public endpoints (`/health`, `/status`) don't need auth

## 📚 Dependencies

Key crates:

- **axum** - Web framework
- **tokio** - Async runtime
- **sqlx** - Type-safe SQL with auto-migrations
- **qdrant-client** - Vector DB client
- **async-openai** - LLM API client (OpenRouter)
- **vosk** - Local speech recognition (fast, offline)
- **reqwest** - HTTP client (ElevenLabs API)
- **serde** - Serialization
- **tracing** - Structured logging

**Voice Stack:**

- Vosk model: `vosk-model-small-en-us-0.15` (~40MB, baked into Docker)
- Audio: 16kHz mono WAV input, MP3 output
- Sessions: In-memory HashMap with background cleanup task

## 🎯 Next: Add Conversation Endpoints

Ready to implement in `handlers/conversation.rs`:

```rust
POST /api/v1/conversations              # Create chat
GET  /api/v1/conversations/{id}         # Get chat
POST /api/v1/conversations/{id}/messages # Send message
```

See database schema tables for structure.

---

**Questions?** Check error logs: `docker-compose logs -f api`

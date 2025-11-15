# Rusty Tea 🍵

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)
![Status](https://img.shields.io/badge/status-Active-brightgreen)

Voice chat AI assistant with speech-to-text (Vosk), LLM chat (OpenRouter), and text-to-speech (ElevenLabs). Ephemeral session-based conversations with PostgreSQL + Qdrant RAG.

**📖 Detailed docs:** See `DEVELOPMENT.md`

---

## ⚡ Quick Start

```bash
# Docker (recommended)
docker-compose -f docker/docker-compose.yml up -d

# Local (need PostgreSQL + Qdrant running)
cargo run
```

Check: `curl http://localhost:3000/health`

---

## 📋 Infrastructure

| Service        | Port | Purpose                            |
| -------------- | ---- | ---------------------------------- |
| **API**        | 3000 | Transcription, chat, LLM           |
| **PostgreSQL** | 5432 | Conversations, messages, documents |
| **Qdrant**     | 6333 | Vector embeddings for RAG          |

**Database:** Auto-migrations run on startup from `migrations/` folder.

---

## 🔐 Authentication

All endpoints except `/health` and `/status` require Bearer token:

```bash
curl -H "Authorization: Bearer your_token" \
  http://localhost:3000/api/v1/transcriptions
```

Set `BEARER_TOKEN` in `.env` (dev) or `docker-compose.yml` (prod).

---

## 📡 Endpoints

| Method | Path                        | Purpose                         |
| ------ | --------------------------- | ------------------------------- |
| GET    | `/health`                   | Health check                    |
| GET    | `/status`                   | Server status + endpoints       |
| POST   | `/api/v1/transcriptions`    | Batch transcription (16kHz WAV) |
| WS     | `/api/v1/transcribe/stream` | Streaming transcription         |
| POST   | `/voice-chat`               | Voice chat (audio in → MP3 out) |

---

## 🧪 Testing

```bash
cargo test                          # All tests
cargo test --release               # Like Docker build
cargo test container_integration_tests  # Container tests
```

---

## 📁 Structure

```
src/
├── main.rs              # Entry, routing, service init
├── config.rs            # Environment variables
├── models.rs            # DTOs
├── middleware.rs        # API key auth
├── handlers/            # HTTP endpoints
└── services/            # Business logic (vosk, database, qdrant, llm)

migrations/              # Auto-run SQL migrations
tests/                   # Unit + integration tests
docker/                  # Dockerfile + docker-compose.yml
```

---

## ✅ Implemented

- ✅ Voice chat with Vosk (local speech recognition)
- ✅ ElevenLabs TTS (text-to-speech MP3 responses)
- ✅ Ephemeral sessions (30min TTL, in-memory)
- ✅ Speech-to-text (batch + streaming WebSocket)
- ✅ OpenRouter LLM with Tea personality
- ✅ Multi-container Docker (PostgreSQL + Qdrant + API)
- ✅ Auto-migrations + Bearer auth

---

## 🔧 Environment Variables

Required in `.env`:

```bash
# Auth
BEARER_TOKEN=your_bearer_token

# Database
DATABASE_URL=postgresql://app@postgres:5432/rusty_tea_db

# LLM (OpenRouter)
OPENROUTER_API_KEY=sk-or-v1-...
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_CHAT_MODEL_LITE=meta-llama/llama-3.1-8b-instruct

# TTS (ElevenLabs)
ELEVENLABS_API_KEY=sk_...
ELEVENLABS_VOICE_ID=EGNfK8LKuwEbqjx3yWz1

# Vector DB
QDRANT_URL=http://qdrant:6333

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info
```

---

## 🛠️ Common Tasks

**Docker commands:**

```bash
docker-compose -f docker/docker-compose.yml up -d      # Start
docker-compose -f docker/docker-compose.yml logs -f    # Logs
docker-compose -f docker/docker-compose.yml down       # Stop
```

**Manual migrations (if needed):**

```bash
cargo install sqlx-cli
sqlx migrate run --database-url "postgresql://postgres:postgres_dev_password@localhost:5432/rusty_tea_db"
```

**Code formatting:**

```bash
cargo fmt
cargo clippy
```

---

See `DEVELOPMENT.md` for detailed setup, environment variables, database schema, code examples, troubleshooting, and more.

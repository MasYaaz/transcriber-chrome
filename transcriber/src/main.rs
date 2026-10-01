mod transcriber;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use transcriber::Transcriber;

const STEP_SAMPLES: usize = 8000;       // 0.5 detik
const MAX_CONTEXT_SAMPLES: usize = 112000; // 7 detik konteks

#[tokio::main]
async fn main() {
    let model_path = "models/ggml-large-v3-turbo-q8_0.bin";

    let transcriber = match Transcriber::new(model_path) {
        Ok(t) => Arc::new(Mutex::new(t)),
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            return;
        }
    };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(Extension(transcriber));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8088));
    println!("🚀 WebSocket Server berjalan di ws://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(transcriber): Extension<Arc<Mutex<Transcriber>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, transcriber))
}

async fn handle_socket(mut socket: WebSocket, transcriber: Arc<Mutex<Transcriber>>) {
    println!("✅ Perekaman dimulai (Chrome Extension terhubung)");

    let mut audio_buffer: Vec<f32> = Vec::new();
    let mut unprocessed_samples = 0;

    // 📝 Penampung seluruh teks transkrip rapat di memori backend
    let mut full_transcript = String::new();

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Binary(bytes) => {
                let new_samples: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();

                let samples_count = new_samples.len();
                audio_buffer.extend(new_samples);
                unprocessed_samples += samples_count;

                if unprocessed_samples >= STEP_SAMPLES {
                    if audio_buffer.len() > MAX_CONTEXT_SAMPLES {
                        let overflow = audio_buffer.len() - MAX_CONTEXT_SAMPLES;
                        audio_buffer.drain(0..overflow);
                    }

                    let samples_to_process = audio_buffer.clone();
                    let transcriber_clone = Arc::clone(&transcriber);

                    let result = tokio::task::spawn_blocking(move || {
                        let mut guard = transcriber_clone.blocking_lock();
                        guard.transcribe(&samples_to_process)
                    })
                    .await;

                    if let Ok(Ok(text)) = result {
                        if !text.is_empty() {
                            // Simpan/gabungkan ke memori backend
                            println!("📝 [Recorded]: {}", text);
                            full_transcript.push_str(&text);
                            full_transcript.push(' ');

                            // ❌ TIDAK PERLU dikirim via socket.send(Message::Text(...)) ke ekstensi
                        }
                    }

                    unprocessed_samples = 0;
                }
            }
            Message::Text(cmd) => {
                // Contoh: Ekstensi mengirim perintah "SUMMARIZE" saat tombol Stop/Summarize diklik
                if cmd == "SUMMARIZE" {
                    println!("🤖 Memproses ringkasan AI untuk transkrip rapat...");
                    // TODO: Panggil modul Agnostic AI Summarizer (Ollama / OpenAI API)
                }
            }
            Message::Close(_) => {
                println!("⏹️ Perekaman selesai.");
                println!("📑 Total Transkrip Terkumpul:\n{}", full_transcript.trim());
                break;
            }
            _ => {}
        }
    }
}

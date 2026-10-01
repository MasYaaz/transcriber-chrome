use std::path::Path;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub struct Transcriber {
    _ctx: WhisperContext,
    state: WhisperState,
}

impl Transcriber {
    pub fn new(model_path: &str) -> Result<Self, String> {
        let path = Path::new(model_path);
        if !path.exists() {
            return Err(format!("Model tidak ditemukan di path: {}", model_path));
        }

        let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| format!("Gagal inisialisasi WhisperContext: {}", e))?;

        // Inisialisasi state SEKALI SAJA di awal
        let state = ctx
            .create_state()
            .map_err(|e| format!("Gagal membuat state: {}", e))?;

        Ok(Self { _ctx: ctx, state })
    }

    pub fn transcribe(&mut self, pcm_samples: &[f32]) -> Result<String, String> {
        // Setup parameter cepat untuk streaming real-time
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_timestamps(false);
        params.set_single_segment(true); // Optimal untuk chunk audio pendek
        params.set_no_context(true); // Mencegah looping halusinasi dari teks sebelumnya

        // FILTER SUARA HENING
        params.set_no_speech_thold(0.4);

        // AMBANG BATAS KELAYAKAN TEKS (Entropy & Logprob Threshold)
        params.set_logprob_thold(-0.8);
        params.set_entropy_thold(2.2);

        // PENANGANAN REPETISI / LOOPING TEKS
        params.set_suppress_blank(true);

        // OPTIMASI THREADS
        params.set_n_threads(1);

        // Eksekusi inferensi langsung dari RAM/VRAM
        self.state
            .full(params, pcm_samples)
            .map_err(|e| format!("Gagal memproses audio PCM: {}", e))?;

        // Ambil teks dari segmen hasil
        let mut text = String::new();
        for segment in self.state.as_iter() {
            if let Ok(segment_text) = segment.to_str_lossy() {
                text.push_str(&segment_text);
                text.push(' ');
            }
        }

        Ok(text.trim().to_string())
    }
}

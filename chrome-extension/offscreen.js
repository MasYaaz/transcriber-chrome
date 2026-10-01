let socket = null;
let audioContext = null;
let mediaStreamSource = null;
let processorNode = null;

chrome.runtime.onMessage.addListener((message) => {
  if (message.action === "INIT_AUDIO_CAPTURE") {
    startAudioPipeline(message.streamId);
  } else if (message.action === "STOP_AUDIO_CAPTURE") {
    stopAudioPipeline();
  }
});

async function startAudioPipeline(streamId) {
  // 1. Hubungkan WebSocket ke Backend Rust
  socket = new WebSocket("ws://127.0.0.1:8088/ws");

  socket.onmessage = (event) => {
    // Kirim teks transkrip dari Rust ke Background Script
    chrome.runtime.sendMessage({
      action: "TRANSCRIPT_RECEIVED",
      text: event.data,
    });
  };

  // 2. Tangkap stream audio menggunakan Stream ID dari tab capture
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      mandatory: {
        chromeMediaSource: "tab",
        chromeMediaSourceId: streamId,
      },
    },
    video: false,
  });

  // Pasang audio output agar suara tab (Meet/YouTube) tetap terdengar di speaker PC
  const audioOutput = new Audio();
  audioOutput.srcObject = stream;
  audioOutput.play();

  // 3. Resample Audio ke 16000Hz Mono Float32
  audioContext = new AudioContext({ sampleRate: 16000 });
  mediaStreamSource = audioContext.createMediaStreamSource(stream);
  processorNode = audioContext.createScriptProcessor(4096, 1, 1);

  processorNode.onaudioprocess = (e) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      const pcmData = e.inputBuffer.getChannelData(0); // Float32Array
      socket.send(pcmData.buffer); // Send Binary Payload ke Rust
    }
  };

  mediaStreamSource.connect(processorNode);
  processorNode.connect(audioContext.destination);
}

function stopAudioPipeline() {
  if (processorNode) processorNode.disconnect();
  if (mediaStreamSource) mediaStreamSource.disconnect();
  if (audioContext) audioContext.close();
  if (socket) socket.close();
}

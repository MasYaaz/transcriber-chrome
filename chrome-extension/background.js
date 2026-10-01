let offscreenCreated = false;

// Pastikan offscreen document siap untuk merekam audio
async function setupOffscreenDocument() {
  if (await chrome.offscreen.hasDocument()) return;

  await chrome.offscreen.createDocument({
    url: "offscreen.html",
    reasons: ["USER_MEDIA"],
    justification:
      "Merekam audio tab untuk transkripsi real-time lokal via Rust.",
  });
}

// Kirim sinyal ke Content Script di tab aktif
async function sendToContentScript(data) {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (tab?.id) {
    chrome.tabs.sendMessage(tab.id, data).catch(() => {});
  }
}

// Handle pesan dari Popup atau Offscreen
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === "START_RECORDING") {
    (async () => {
      // Ambil stream ID dari tab aktif saat ini
      const [tab] = await chrome.tabs.query({
        active: true,
        currentWindow: true,
      });
      if (!tab) return;

      const streamId = await chrome.tabCapture.getMediaStreamId({
        targetTabId: tab.id,
      });
      await setupOffscreenDocument();

      // Perintahkan offscreen document untuk mulai menangkap audio via streamId
      chrome.runtime.sendMessage({
        action: "INIT_AUDIO_CAPTURE",
        streamId: streamId,
      });

      chrome.storage.local.set({ isRecording: true });
      sendToContentScript({ action: "SHOW_OVERLAY" });
      sendResponse({ status: "STARTED" });
    })();
    return true; // async sendResponse
  }

  if (message.action === "STOP_RECORDING") {
    chrome.runtime.sendMessage({ action: "STOP_AUDIO_CAPTURE" });
    chrome.storage.local.set({ isRecording: false });
    sendToContentScript({ action: "HIDE_OVERLAY" });
    sendResponse({ status: "STOPPED" });
    return true;
  }

  if (message.action === "TRANSCRIPT_RECEIVED") {
    // Teruskan teks hasil transkrip ke Content Script (Overlay Subtitle)
    sendToContentScript({
      action: "UPDATE_TRANSCRIPT",
      text: message.text,
    });
  }
});

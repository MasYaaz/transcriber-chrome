const btnAction = document.getElementById("btnAction");
const dotStatus = document.getElementById("dotStatus");
const textStatus = document.getElementById("textStatus");

// Sinkronkan UI dengan status perekaman di storage saat popup dibuka
chrome.storage.local.get(["isRecording"], (result) => {
  updateUI(result.isRecording || false);
});

btnAction.addEventListener("click", () => {
  chrome.storage.local.get(["isRecording"], (result) => {
    const isRecording = result.isRecording || false;

    if (!isRecording) {
      chrome.runtime.sendMessage({ action: "START_RECORDING" }, (res) => {
        if (res?.status === "STARTED") updateUI(true);
      });
    } else {
      chrome.runtime.sendMessage({ action: "STOP_RECORDING" }, (res) => {
        if (res?.status === "STOPPED") updateUI(false);
      });
    }
  });
});

function updateUI(isRecording) {
  if (isRecording) {
    btnAction.className = "btn btn-stop";
    btnAction.innerHTML = "<span>⏹ Stop Recording</span>";
    dotStatus.className = "dot active";
    textStatus.innerText = "Merekam di Background...";
  } else {
    btnAction.className = "btn btn-start";
    btnAction.innerHTML = "<span>▶ Start Recording</span>";
    dotStatus.className = "dot";
    textStatus.innerText = "Siap Merekam";
  }
}

import init, {
  WasmAngleReliability,
  fashionstar_decode_angle,
  fashionstar_query_angle_packet,
} from "./pkg/smart_servo_wasm.js";

const canvas = document.querySelector("#chart");
const ctx = canvas.getContext("2d");
const rawEl = document.querySelector("#raw");
const filteredEl = document.querySelector("#filtered");
const reliableEl = document.querySelector("#reliable");
const statusEl = document.querySelector("#status");
const txCountEl = document.querySelector("#tx-count");
const rxCountEl = document.querySelector("#rx-count");
const debugLogEl = document.querySelector("#debug-log");
const connectBtn = document.querySelector("#connect");
const disconnectBtn = document.querySelector("#disconnect");
const servoIdEl = document.querySelector("#servo-id");
const baudrateEl = document.querySelector("#baudrate");
const multiTurnEl = document.querySelector("#multi-turn");
const zeroHoldEl = document.querySelector("#zero-hold-s");

const yMin = -180;
const yMax = 180;
const maxPoints = 180;
const pollIntervalMs = 20;
const responseTimeoutMs = 120;
let filter;
let points = [];
let port;
let reader;
let writer;
let live = false;
let rxBuffer = new Uint8Array(0);
let txCount = 0;
let rxCount = 0;
let lastRxAt = 0;
let debugLines = [];

function setStatus(text) {
  statusEl.textContent = text;
}

function logDebug(line) {
  const stamp = new Date().toLocaleTimeString();
  debugLines.push(`[${stamp}] ${line}`);
  debugLines = debugLines.slice(-12);
  debugLogEl.textContent = debugLines.join("\n");
}

function bytesToHex(bytes, max = 48) {
  const view = Array.from(bytes.slice(0, max));
  const suffix = bytes.length > max ? ` ... +${bytes.length - max}` : "";
  return view.map((byte) => byte.toString(16).padStart(2, "0")).join(" ") + suffix;
}

function setCounters() {
  txCountEl.textContent = String(txCount);
  rxCountEl.textContent = String(rxCount);
}

function zeroConfirmSamples() {
  const seconds = Number(zeroHoldEl.value);
  return Math.max(1, Math.round((seconds * 1000) / pollIntervalMs));
}

function yFor(value) {
  const t = (value - yMin) / (yMax - yMin);
  return canvas.height - t * canvas.height;
}

function xFor(index) {
  if (maxPoints <= 1) return 0;
  return (index / (maxPoints - 1)) * canvas.width;
}

function drawGrid() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#101620";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.strokeStyle = "rgba(255,255,255,0.10)";
  ctx.lineWidth = 1;
  for (let deg = -180; deg <= 180; deg += 45) {
    const y = yFor(deg);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(canvas.width, y);
    ctx.stroke();
    ctx.fillStyle = "rgba(255,255,255,0.55)";
    ctx.fillText(`${deg} deg`, 10, y - 4);
  }
}

function drawLine(key, color, width) {
  if (points.length < 2) return;
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.beginPath();
  points.forEach((point, index) => {
    const x = xFor(index);
    const y = yFor(point[key]);
    if (index === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();
}

function render() {
  drawGrid();
  drawLine("raw", "#54a8ff", 2);
  drawLine("filtered", "#ffca5a", 3);

  const latest = points.at(-1);
  rawEl.textContent = latest ? `${latest.raw.toFixed(3)} deg` : "--";
  filteredEl.textContent = latest ? `${latest.filtered.toFixed(3)} deg` : "--";
  reliableEl.textContent = latest ? String(latest.reliable) : "--";
}

function pushRaw(raw) {
  const sample = filter.filter(raw);
  points.push({ raw: sample.raw_deg, filtered: sample.filtered_deg, reliable: sample.reliable });
  if (points.length > maxPoints) points = points.slice(-maxPoints);
  render();
}

function appendBytes(left, right) {
  const out = new Uint8Array(left.length + right.length);
  out.set(left, 0);
  out.set(right, left.length);
  return out;
}

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForResponse(servoId, multiTurn, startedAt) {
  while (performance.now() - startedAt < responseTimeoutMs) {
    const decoded = fashionstar_decode_angle(rxBuffer, servoId, multiTurn);
    if (decoded.found) return decoded;
    if (decoded.error) return decoded;
    await sleep(4);
  }
  return null;
}

async function serialReadLoop() {
  while (live && port?.readable) {
    reader = port.readable.getReader();
    try {
      while (live) {
        const { value, done } = await reader.read();
        if (done) break;
        if (value?.length) {
          rxCount += value.length;
          lastRxAt = performance.now();
          rxBuffer = appendBytes(rxBuffer, value);
          if (rxBuffer.length > 512) rxBuffer = rxBuffer.slice(-512);
          setCounters();
          logDebug(`RX ${value.length} bytes: ${bytesToHex(value)}`);
        }
      }
    } catch (error) {
      if (live) setStatus(`read error: ${error.message}`);
    } finally {
      reader.releaseLock();
      reader = undefined;
    }
  }
}

async function serialPollLoop() {
  while (live && port?.writable) {
    const servoId = Number(servoIdEl.value);
    const multiTurn = multiTurnEl.checked;
    const packet = fashionstar_query_angle_packet(servoId, multiTurn);
    rxBuffer = new Uint8Array(0);
    await writer.write(packet);
    txCount += packet.length;
    setCounters();
    logDebug(`TX query id=${servoId} multi=${multiTurn}: ${bytesToHex(packet)}`);

    const decoded = await waitForResponse(servoId, multiTurn, performance.now());
    if (decoded?.found) {
      pushRaw(decoded.raw_deg);
      setStatus(`live WebSerial, last RX ${Math.round(performance.now() - lastRxAt)}ms ago`);
      logDebug(`DECODE raw=${decoded.raw_deg.toFixed(3)} deg`);
    } else if (decoded?.error) {
      setStatus(decoded.error);
      logDebug(`DECODE error: ${decoded.error}`);
    } else {
      setStatus(`timeout waiting for id=${servoId}; check id, baudrate, power, TX/RX wiring`);
      logDebug("TIMEOUT no valid FashionStar angle response");
    }

    await sleep(pollIntervalMs);
  }
}

async function connectSerial() {
  if (!("serial" in navigator)) {
    setStatus("WebSerial requires Chrome/Edge over localhost or HTTPS");
    return;
  }

  reset();
  txCount = 0;
  rxCount = 0;
  debugLines = [];
  setCounters();
  const baudRate = Number(baudrateEl.value);
  port = await navigator.serial.requestPort();
  await port.open({ baudRate, dataBits: 8, stopBits: 1, parity: "none", flowControl: "none" });
  writer = port.writable.getWriter();
  live = true;
  rxBuffer = new Uint8Array(0);
  connectBtn.disabled = true;
  disconnectBtn.disabled = false;
  setStatus(`connected @ ${baudRate}`);
  logDebug(`OPEN baud=${baudRate}`);
  void serialReadLoop();
  void serialPollLoop();
}

async function disconnectSerial() {
  live = false;
  try {
    await reader?.cancel();
  } catch (_) {
    // Ignore cancellation races during disconnect.
  }
  if (writer) {
    writer.releaseLock();
    writer = undefined;
  }
  if (port) {
    await port.close();
    port = undefined;
  }
  connectBtn.disabled = false;
  disconnectBtn.disabled = true;
  setStatus("disconnected");
  logDebug("CLOSE");
}

async function play(values, delayMs = 50) {
  if (live) await disconnectSerial();
  setStatus("simulation mode");
  for (const value of values) {
    pushRaw(value);
    await sleep(delayMs);
  }
}

function reset() {
  filter = WasmAngleReliability.with_config(1.0, 20.0, zeroConfirmSamples());
  points = [];
  render();
}

function glitchSequence() {
  const stableA = Array(35).fill(-72);
  const badZero = Array(20).fill(0);
  const stableB = [-68, -62, -55, -48, -42, -36, -30, -28, -27, -27, -27, -27];
  return [...stableA, ...badZero, ...stableB];
}

function realZeroSequence() {
  const stableA = Array(25).fill(-64);
  const zeros = Array(40).fill(0);
  return [...stableA, ...zeros];
}

await init();
reset();
setCounters();

connectBtn.addEventListener("click", () => connectSerial().catch((error) => setStatus(error.message)));
disconnectBtn.addEventListener("click", () => disconnectSerial().catch((error) => setStatus(error.message)));
document.querySelector("#play-glitch").addEventListener("click", () => play(glitchSequence()));
document.querySelector("#play-zero").addEventListener("click", () => play(realZeroSequence()));
document.querySelector("#clear").addEventListener("click", reset);
zeroHoldEl.addEventListener("change", () => {
  reset();
  setStatus(`zero hold set to ${zeroHoldEl.value}s`);
});

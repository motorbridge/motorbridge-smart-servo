import init, {
  WasmAngleReliability,
  fashionstar_sync_monitor_packet,
  fashionstar_count_monitor_packets,
  fashionstar_decode_monitor_angle,
} from "./pkg/smart_servo_wasm.js";

// ── palette ──────────────────────────────────────────────────────────────────
const COLORS = ["#54a8ff", "#ffca5a", "#5af278", "#ff7eb3", "#c87bff", "#ff9b5a", "#5af0ff"];

// ── DOM refs ─────────────────────────────────────────────────────────────────
const canvas = document.getElementById("chart");
const ctx = canvas.getContext("2d");
const sidebar = document.getElementById("sidebar");
const statusEl = document.getElementById("status");
const connectBtn = document.getElementById("connect");
const disconnectBtn = document.getElementById("disconnect");
const baudrateEl = document.getElementById("baudrate");
const idsEl = document.getElementById("ids");
const historyEl = document.getElementById("history-s");

// ── state ────────────────────────────────────────────────────────────────────
let port,
  reader,
  writer,
  live = false;
let rxBuffer = new Uint8Array(0);

// per-servo
let servoIds = [];
let filters = {}; // id → WasmAngleReliability
let history = {}; // id → [{ts, angle, reliable, voltage}]
let cards = {}; // id → DOM element

const POLL_MS = 20; // target ~50 Hz
const RESPONSE_WAIT = 100; // max wait for all responses (ms)
const HISTORY_MS = () => Number(historyEl.value) * 1000;

// ── helpers ───────────────────────────────────────────────────────────────────
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function setStatus(t) {
  statusEl.textContent = t;
}

function getIds() {
  return idsEl.value
    .trim()
    .split(/\s+/)
    .map(Number)
    .filter((n) => n >= 0 && n <= 253);
}

function appendBytes(a, b) {
  const out = new Uint8Array(a.length + b.length);
  out.set(a);
  out.set(b, a.length);
  return out;
}

// ── sidebar cards ─────────────────────────────────────────────────────────────
function buildCards(ids) {
  sidebar.innerHTML = "";
  cards = {};
  for (const id of ids) {
    const color = COLORS[id % COLORS.length];
    const card = document.createElement("div");
    card.className = "servo-card no-data";
    card.style.setProperty("--color", color);
    card.innerHTML = `
      <div class="header">
        <div class="dot"></div>
        <span class="id">ID ${id}</span>
      </div>
      <div class="angle">--.-°</div>
      <div class="meta">-- V · --</div>
    `;
    sidebar.appendChild(card);
    cards[id] = card;
  }
}

function updateCard(id, angle, reliable, voltage) {
  const card = cards[id];
  if (!card) return;
  const color = COLORS[id % COLORS.length];
  card.className = "servo-card " + (reliable ? "reliable" : "unreliable");
  card.style.setProperty("--color", color);
  card.querySelector(".angle").textContent = `${angle.toFixed(2)}°`;
  card.querySelector(".meta").textContent = `${(voltage / 1000).toFixed(2)} V · ${reliable ? "ok" : "~"}`;
}

// ── canvas chart ──────────────────────────────────────────────────────────────
function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width * devicePixelRatio;
  canvas.height = rect.height * devicePixelRatio;
  canvas.style.width = rect.width + "px";
  canvas.style.height = rect.height + "px";
}

function drawChart() {
  const W = canvas.width,
    H = canvas.height;
  const dpr = devicePixelRatio;

  ctx.clearRect(0, 0, W, H);
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);

  const now = Date.now();
  const windowMs = HISTORY_MS();

  // collect y range across all series
  let yMin = Infinity,
    yMax = -Infinity;
  for (const id of servoIds) {
    for (const pt of history[id]) {
      if (now - pt.ts <= windowMs) {
        yMin = Math.min(yMin, pt.angle);
        yMax = Math.max(yMax, pt.angle);
      }
    }
  }
  if (!isFinite(yMin)) {
    yMin = -180;
    yMax = 180;
  }
  const pad = Math.max((yMax - yMin) * 0.12, 5);
  yMin -= pad;
  yMax += pad;

  const xOf = (ts) => ((ts - (now - windowMs)) / windowMs) * W;
  const yOf = (v) => H - ((v - yMin) / (yMax - yMin)) * H;

  // grid
  ctx.strokeStyle = "rgba(255,255,255,0.06)";
  ctx.lineWidth = 1;
  const gridStep = niceStep(yMax - yMin);
  const gridStart = Math.ceil(yMin / gridStep) * gridStep;
  ctx.font = `${10 * dpr}px ui-monospace, monospace`;
  ctx.fillStyle = "rgba(255,255,255,0.3)";
  for (let g = gridStart; g <= yMax; g += gridStep) {
    const y = yOf(g);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(W, y);
    ctx.stroke();
    ctx.fillText(`${g.toFixed(0)}°`, 6 * dpr, y - 3 * dpr);
  }

  // series
  for (const id of servoIds) {
    const pts = history[id].filter((p) => now - p.ts <= windowMs);
    if (pts.length < 2) continue;
    const color = COLORS[id % COLORS.length];

    // draw segments, switching dash style on reliable changes
    let i = 0;
    while (i < pts.length) {
      const rel = pts[i].reliable;
      let j = i + 1;
      while (j < pts.length && pts[j].reliable === rel) j++;

      ctx.beginPath();
      ctx.strokeStyle = rel ? color : "rgba(255,255,255,0.18)";
      ctx.lineWidth = rel ? 2 * dpr : 1.5 * dpr;
      ctx.setLineDash(rel ? [] : [6 * dpr, 4 * dpr]);

      for (let k = i; k < j; k++) {
        const x = xOf(pts[k].ts),
          y = yOf(pts[k].angle);
        k === i ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.stroke();
      i = j;
    }
    ctx.setLineDash([]);

    // label at end of line
    const last = pts.at(-1);
    if (last) {
      ctx.fillStyle = color;
      ctx.fillText(`ID${id}`, xOf(last.ts) + 4 * dpr, yOf(last.angle));
    }
  }
}

function niceStep(range) {
  const raw = range / 5;
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const norm = raw / mag;
  const nice = norm < 1.5 ? 1 : norm < 3 ? 2 : norm < 7 ? 5 : 10;
  return nice * mag;
}

// ── polling ───────────────────────────────────────────────────────────────────
async function pollLoop() {
  while (live) {
    const ids = servoIds;
    const packet = fashionstar_sync_monitor_packet(new Uint8Array(ids));
    rxBuffer = new Uint8Array(0);

    try {
      await writer.write(packet);
    } catch (e) {
      setStatus(`write error: ${e.message}`);
      break;
    }

    // wait until all responses arrive or timeout
    const deadline = performance.now() + RESPONSE_WAIT;
    while (performance.now() < deadline) {
      if (fashionstar_count_monitor_packets(rxBuffer) >= ids.length) break;
      await sleep(4);
    }

    const now = Date.now();
    for (const id of ids) {
      const result = fashionstar_decode_monitor_angle(rxBuffer, id);
      if (result.found) {
        const sample = filters[id].filter(result.raw_deg);
        history[id].push({
          ts: now,
          angle: sample.filtered_deg,
          reliable: sample.reliable,
          voltage: result.voltage_mv,
        });
        // trim old points
        const cutoff = now - HISTORY_MS() - 500;
        history[id] = history[id].filter((p) => p.ts >= cutoff);
        updateCard(id, sample.filtered_deg, sample.reliable, result.voltage_mv);
      }
    }

    drawChart();

    const elapsed = performance.now() - (deadline - RESPONSE_WAIT);
    const remain = POLL_MS - elapsed;
    if (remain > 0) await sleep(remain);
  }
}

async function readLoop() {
  while (live && port?.readable) {
    reader = port.readable.getReader();
    try {
      while (live) {
        const { value, done } = await reader.read();
        if (done) break;
        if (value?.length) {
          rxBuffer = appendBytes(rxBuffer, value);
          if (rxBuffer.length > 1024) rxBuffer = rxBuffer.slice(-1024);
        }
      }
    } catch (_) {
      // port closed or cancelled
    } finally {
      reader.releaseLock();
      reader = undefined;
    }
  }
}

// ── connect / disconnect ──────────────────────────────────────────────────────
async function connect() {
  if (!("serial" in navigator)) {
    setStatus("WebSerial requires Chrome or Edge (localhost or HTTPS)");
    return;
  }

  servoIds = getIds();
  if (!servoIds.length) {
    setStatus("no valid servo IDs");
    return;
  }

  // reset per-servo state
  filters = {};
  history = {};
  for (const id of servoIds) {
    filters[id] = new WasmAngleReliability();
    history[id] = [];
  }
  buildCards(servoIds);

  const baudRate = Number(baudrateEl.value);
  port = await navigator.serial.requestPort();
  await port.open({ baudRate, dataBits: 8, stopBits: 1, parity: "none", flowControl: "none" });
  writer = port.writable.getWriter();
  live = true;

  connectBtn.disabled = true;
  disconnectBtn.disabled = false;
  setStatus(`connected @ ${baudRate} · IDs [${servoIds.join(", ")}]`);

  void readLoop();
  void pollLoop();
}

async function disconnect() {
  live = false;
  try {
    await reader?.cancel();
  } catch (_) {}
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
}

// ── init ──────────────────────────────────────────────────────────────────────
await init();

resizeCanvas();
drawChart();
window.addEventListener("resize", () => {
  resizeCanvas();
  drawChart();
});

connectBtn.addEventListener("click", () => connect().catch((e) => setStatus(e.message)));
disconnectBtn.addEventListener("click", () => disconnect().catch((e) => setStatus(e.message)));
document.getElementById("clear").addEventListener("click", () => {
  for (const id of servoIds) {
    history[id] = [];
    filters[id] = new WasmAngleReliability();
  }
  drawChart();
});
historyEl.addEventListener("change", () => drawChart());

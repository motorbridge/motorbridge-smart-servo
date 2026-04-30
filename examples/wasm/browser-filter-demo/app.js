import init, { WasmAngleReliability } from "./pkg/smart_servo_wasm.js";

const canvas = document.querySelector("#chart");
const ctx = canvas.getContext("2d");
const rawEl = document.querySelector("#raw");
const filteredEl = document.querySelector("#filtered");
const reliableEl = document.querySelector("#reliable");

const yMin = -180;
const yMax = 180;
const maxPoints = 180;
let filter;
let points = [];

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

async function play(values, delayMs = 50) {
  for (const value of values) {
    pushRaw(value);
    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }
}

function reset() {
  filter = new WasmAngleReliability();
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

document.querySelector("#play-glitch").addEventListener("click", () => play(glitchSequence()));
document.querySelector("#play-zero").addEventListener("click", () => play(realZeroSequence()));
document.querySelector("#clear").addEventListener("click", reset);

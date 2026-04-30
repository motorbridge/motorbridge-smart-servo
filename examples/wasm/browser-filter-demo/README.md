# Browser WASM Filter Demo

This demo runs the Rust `AngleReliability` filter in the browser through `wasm-bindgen`.

It visualizes two curves:

- `raw`: incoming protocol angle samples
- `filtered`: the value after suppressing power-cycle `A -> 0 -> B` glitches

## Build the WASM binding

From the repository root:

```powershell
examples\wasm\browser-filter-demo\build.ps1
```

On Linux/macOS:

```bash
bash examples/wasm/browser-filter-demo/build.sh
```

## Run

Serve the directory over HTTP. ES modules and `.wasm` loading usually do not work from `file://`.

```powershell
cd examples\wasm\browser-filter-demo
python -m http.server 8080
```

Open:

```text
http://localhost:8080
```

## Using release assets instead

You can also download `motorbridge-smart-servo-wasm.tar.gz` from GitHub Releases and extract it into this demo directory as `pkg/` if it contains the generated wasm-bindgen files.

## Hardware note

This demo does not open a serial port. Browser UART control needs WebSerial or a native bridge. A future browser transport can read raw angle samples from WebSerial and pass them through:

```js
const sample = filter.filter(rawAngleDeg);
console.log(sample.raw_deg, sample.filtered_deg, sample.reliable);
```

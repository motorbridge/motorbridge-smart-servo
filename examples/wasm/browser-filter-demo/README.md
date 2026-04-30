# Browser WASM + WebSerial Demo

This demo runs FashionStar query/decode logic and the Rust `AngleReliability` filter in the browser through `wasm-bindgen`.

It visualizes two curves:

- `raw`: angle decoded from real FashionStar protocol responses, or from simulation buttons
- `filtered`: the value after suppressing power-cycle `A -> 0 -> B` glitches

## What runs where

- WebSerial JavaScript opens and reads/writes the serial port.
- WASM builds the FashionStar query-angle packet.
- WASM decodes FashionStar response packets.
- WASM filters raw angles into safe filtered angles.

The browser must handle serial I/O because native Rust `serialport` cannot run inside browser WASM.

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

Serve the directory over HTTP. WebSerial is available only in secure contexts, and `localhost` counts as secure.

```powershell
cd examples\wasm\browser-filter-demo
python -m http.server 8080
```

Open in Chrome or Edge:

```text
http://localhost:8080
```

Click `Connect WebSerial`, choose the USB serial adapter, and keep the defaults for a FashionStar bus:

- baudrate: `1000000`
- servo id: `0`
- multi-turn: checked

## Simulation mode

The simulation buttons still work without hardware. They are useful for checking the filter behavior before connecting a real servo.

## Using release assets instead

You can also download `motorbridge-smart-servo-wasm.tar.gz` from GitHub Releases and extract it into this demo directory as `pkg/` if it contains the generated wasm-bindgen files.

# GazeRest

GazeRest is a lightweight desktop helper for the 20-20-20 eye rest habit. It runs locally, stays in the system tray, and gently reminds you to look away from the screen after continuous screen time.

## Features

- Offline desktop app built with Tauri 2, React, and TypeScript.
- Local reminder scheduling, break countdown, tray controls, and settings persistence.
- Four reminder levels from status-only to immersive reminders.
- Viewing-distance calculator based on monitor width and height.
- Bilingual interface: Simplified Chinese and English.

## Development

```bash
npm install
npm run tauri:dev
```

## Checks

```bash
npm run test:run
npm run build
cd src-tauri
cargo check
```

## Release Build

```bash
npm run tauri:build
```

For a quick Windows smoke build without creating installers:

```bash
npx tauri build --no-bundle
```

The generated executable is under `src-tauri/target/release/`.

## Privacy

GazeRest does not upload data, record screen content, or request camera or microphone permissions. Settings and reminder records are stored locally on the device.

# Etcherus

Etcherus is a cross-platform USB flashing tool built in Rust using the [Iced GUI library](https://github.com/iced-rs/iced). It provides a clean, user-friendly interface to flash OS images (`.iso`, `.img`), firmware, and other binary files onto external USB and SD card storage drives.

## Features

- **Cross-Platform Storage Detection**: Uses platform-native utilities and APIs to query connected drives safely without heavy dependencies like `libusb`:
  - **macOS**: Parses `diskutil list external` and `diskutil info`.
  - **Linux**: Queries block devices.
  - **Windows**: Interfaces directly with WinAPI.
- **Safety Checks**: Automatically checks drive capacities against the selected image size to prevent flashing failures and guard against writing to system/internal drives.
- **Async File Picker**: Native, non-blocking file selection dialogs via `rfd` (Rust File Dialogs).
- **Progress Tracking**: Real-time progress updates during the flashing simulation/write phase.

## Project Structure

- `src/main.rs`: Entry point initializing the Iced application loop.
- `src/app.rs`: Core application logic, message handling state machine, and orchestrator.
- `src/models/`:
  - [drive_model.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/models/drive_model.rs): Structs representing target storage drives, capacity sizes, and capabilities.
  - [image_model.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/models/image_model.rs): Structs representing the input source images to be written.
- `src/services/`:
  - [usb_service.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/services/usb_service.rs): Service layer querying disk layout from system tools and APIs.
  - [flash_service.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/services/flash_service.rs): Simulates and runs drive-flashing pipelines.
- `src/views/`:
  - [home_view.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/views/home_view.rs): The main setup workspace where users choose images, select targets, and monitor flash status.
  - [flash_progress_view.rs](file:///Users/datacube/Files/Experiments/rust/etcherus/src/views/flash_progress_view.rs): Component rendering progress bars, status highlights, and outcomes.

## Getting Started

### Prerequisites

You need [Rust and Cargo](https://rustup.rs/) installed on your machine.

### Installation & Run

Clone the repository and run using Cargo:

```bash
cargo run --release
```

## How It Works

1. **Scan**: The application queries system block devices to find external/removable storage drives.
2. **Select Image**: Click **Select Image** to browse for your installation media (`.iso`, `.img`, `.bin`).
3. **Select Target**: Choose one of the detected removable USB/SD drives.
4. **Validation**: Etcherus verifies if the target device is large enough to contain the selected image.
5. **Flash**: Click **Flash** to begin writing the image safely.

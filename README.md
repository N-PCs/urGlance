# urGlance

A high-performance, responsive file manager and organizer utility designed to provide lightning-fast file previews and seamless system integration.

## Features
- **Instant Previews**: Fast file lookups and preview extraction via a background daemon.
- **Native GUI Control Panel**: A fast, cross-platform UI built in Rust using Slint. View your file preview history and manage system autostart.
- **High Performance**: Built with optimization and low latency in mind.

## Platform Support
- **Windows** (Native or via MSYS2/MinGW)
- **Linux** (Any modern distribution)
*(Note: macOS is not supported.)*

## Getting Started

### Prerequisites
- Git
- Rust (Edition 2021)
- C++17 compatible compiler (MSVC, GCC, or Clang)
- Node.js & npm (for web frontend fallback)

### Installation
```bash
git clone https://github.com/N-PCs/urGlance.git
cd urGlance/core-logic
cargo run --release
```

Made by @N-PCs for the world!

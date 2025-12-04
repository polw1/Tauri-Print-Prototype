# Print Prototype

Web-based document editor with printing and PDF export capabilities.

Built with Vue 3, TypeScript, Tauri, and Rust.

## Features

- WYSIWYG editor with multiple editable pages
- PDF export with file save dialog
- Paper format control: A3, A4, Letter, Legal
- Portrait and landscape orientation
- Add and remove pages dynamically
- Real-time preview

## Architecture

### Frontend
- Vue 3 (Composition API)
- TypeScript
- Vite
- Composables for print system and Tauri communication
- Automatic zoom calculation based on viewport size

### Backend
- Rust
- Tauri v2
- headless_chrome for PDF rendering and printing
- Serde for data serialization
- Native print system integration (CUPS on Linux, native APIs on Windows/macOS)

## Installation

### Requirements

- Node.js 18+
- Rust (install via [rustup](https://rustup.rs/))
- Tauri CLI

### Setup

```bash
# Clone repository
git clone <repo-url>
cd print-prototype

# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

### Optional Dependencies

**PDF Export:** No external dependencies required. Uses embedded Chrome headless.

**Direct Printing:**

Linux:
```bash
sudo apt install cups
```

Windows/macOS: System printers work automatically.

## Project Structure

```
print-prototype/
├── src/                        # Vue 3 frontend
│   ├── components/
│   │   ├── PrintPage.vue       # Editable page component
│   │   ├── PrintToolbar.vue    # Toolbar controls
│   │   └── PrinterSelectModal.vue
│   ├── composables/
│   │   ├── usePrintSystem.ts   # Page management and zoom
│   │   └── useTauriPrint.ts    # Rust backend communication
│   └── types/
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/
│   │   │   └── print.rs        # Print and PDF logic
│   │   └── models/
│   └── Cargo.toml
└── package.json
```


## License

MIT



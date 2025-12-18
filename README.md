# TidePerfect

## Overview

TidePerfect is a native desktop application that brings TIDAL's high-fidelity music streaming to linux.

## Installation

### Prerequisites

You'll need TIDAL API credentials to use TidePerfect:

1. Obtain TIDAL API credentials (Client ID and Client Secret) - *these must be extracted from other Tidal applications, credentials created with the Tidal developer portal will not work.*
2. Create a `.env` file in `src-tauri/` with your credentials:

```env
TIDAL_CLIENT_ID=your_client_id
TIDAL_CLIENT_SECRET=your_client_secret
```

### From Source

**Requirements:**
- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/)
- [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/) for your platform

**Build Steps:**

```bash
# Clone the repository
git clone https://github.com/yourusername/tideperfect.git
cd tideperfect

# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```


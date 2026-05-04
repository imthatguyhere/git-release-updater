# Functionality

## Data Flow

```md
main
  ├── util::current_timestamp()
  ├── util::format_timestamp()
  ├── util::truncate()
  ├── util::is_valid_url()
  ├── request::get()
  ├── request::get_json()
  ├── request::post()
  ├── request::post_json()
  └── request::request() / request_json()
```

## Configuration

### _(none yet — skeleton stage)_

## Build and Release Packaging

Standard `cargo build --release`. No custom profile settings yet.

## Modules

### main

- **Purpose:** Binary entry point. Initializes Tokio async runtime, orchestrates module calls.
- **Public:** `fn main()` — async entry via `#[tokio::main]`.

### request

- **Purpose:** HTTP client wrapper. Abstracts `reqwest` for common methods.
- **Types:**
  - `Response` — status code + body string
  - `Method` — enum: Get, Post, Put, Delete
- **Public functions:**
  - `request()` — generic HTTP call with optional JSON body
  - `request_json()` — HTTP call + JSON deserialization
  - `get()` — raw GET
  - `get_json()` — GET with JSON response
  - `post()` — raw POST
  - `post_json()` — POST with JSON response

### util

- **Purpose:** General-purpose helpers.
- **Public functions:**
  - `current_timestamp()` — Unix timestamp in seconds
  - `format_timestamp()` — human-readable date-time string
  - `truncate()` — string truncation with ellipsis
  - `is_valid_url()` — URL prefix check

## Public API

### _(All public items are documented in their module sections above.)_

# 📸 EXIF Metadata API

> A powerful REST API for reading, writing, and deleting image metadata (EXIF, IPTC, XMP, GPS, MakerNotes, C2PA) built in Rust.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-✓-blue.svg)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0.3-6BA539.svg)](docs/openapi.yaml)

---

## 📋 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [API Endpoints](#-api-endpoints)
- [Metadata Types](#-metadata-types)
- [Usage Examples](#-usage-examples)
- [Testing](#-testing)
- [Documentation](#-documentation)
- [Installation](#-installation)
- [Environment Variables](#-environment-variables)
- [FAQ](#-faq)
- [License](#-license)

---

## ✨ Features

- 📖 **Read** all metadata types (EXIF, IPTC, XMP, GPS, MakerNotes)
- ✏️ **Write/update** metadata in images
- 🗑️ **Delete** metadata (including C2PA AI signatures)
- 🗺️ **Change GPS coordinates** and other tags
- 🤖 **Remove AI traces** (C2PA/Content Credentials)
- 📦 Supports **IPTC and XMP** for professional workflows
- 🐳 **Docker-ready** for easy deployment
- ⚡ **High performance** (Rust + Actix-web)
- 🔒 **Secure processing** (temporary files auto-deleted)

---

## 🚀 Quick Start

### Run via Docker (recommended)

```bash
# 1. Clone the repository
git clone https://github.com/your-repo/exif-api-rust.git
cd exif-api-rust

# 2. Build and start the container
./run.sh

# 3. Check that it works
curl http://localhost:3000/health
```

### Local build

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Build the project
cargo build --release

# 3. Run it
./target/release/exif-api
```

---

## 📡 API Endpoints

All endpoints except `/health` accept **POST** requests with `multipart/form-data`.

| Endpoint | Method | Description | Response Format |
|----------|--------|-------------|-----------------|
| `/health` | `GET` | Service health check | JSON |
| `/read` | `POST` | Read metadata | JSON |
| `/write` | `POST` | Write/update metadata | Binary file |
| `/delete` | `POST` | Delete metadata | Binary file |

---

### 1. Health Check

```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "supported_metadata": [
    "all", "exif", "iptc", "xmp", "makernotes",
    "gps", "c2pa", "jumbf", "png"
  ],
  "description": "Use 'type' parameter to filter metadata groups"
}
```

---

### 2. Read Metadata (`/read`)

```bash
curl -s -X POST -F "file=@photo.jpg" http://localhost:3000/read | jq '.'
```

**With `type` parameter:**
```bash
# EXIF only
curl -s -X POST -F "file=@photo.jpg" -F "type=exif" http://localhost:3000/read | jq '.'

# GPS only
curl -s -X POST -F "file=@photo.jpg" -F "type=gps" http://localhost:3000/read | jq '.'
```

---

### 3. Write Metadata (`/write`)

```bash
# Write EXIF tags
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"Make":"Canon","Model":"EOS R5","Artist":"John Doe"}' \
  -F "type=exif" \
  http://localhost:3000/write \
  --output modified.jpg

# Write GPS coordinates (New York, Central Park)
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"GPSLatitude":"40.7812","GPSLongitude":"-73.9665"}' \
  -F "type=gps" \
  http://localhost:3000/write \
  --output with_gps.jpg

# Write IPTC tags
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"Keywords":"photo,travel","City":"New York","Country":"USA"}' \
  -F "type=iptc" \
  http://localhost:3000/write \
  --output with_iptc.jpg
```

---

### 4. Delete Metadata (`/delete`)

```bash
# Delete specific tags
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'tags=["Make","Model","Artist"]' \
  -F "type=exif" \
  http://localhost:3000/delete \
  --output deleted_tags.jpg

# Delete all GPS
curl -X POST -F "file=@photo.jpg" -F "type=gps" http://localhost:3000/delete --output no_gps.jpg

# Remove AI signatures (C2PA)
curl -X POST -F "file=@ai_image.jpg" -F "type=c2pa" http://localhost:3000/delete --output clean.jpg

# Full metadata wipe
curl -X POST -F "file=@photo.jpg" http://localhost:3000/delete --output no_metadata.jpg
```

---

## 📋 Metadata Types

| Type | Description | Example Tags |
|------|-------------|--------------|
| `all` | All metadata | — |
| `exif` | EXIF | `Make`, `Model`, `ISO`, `FNumber`, `ExposureTime` |
| `iptc` | IPTC | `Keywords`, `City`, `Country`, `Caption`, `Credit` |
| `xmp` | XMP | `Creator`, `Rights`, `Description`, `Subject` |
| `gps` | GPS | `GPSLatitude`, `GPSLongitude`, `GPSAltitude` |
| `makernotes` | Manufacturer data | `SerialNumber`, `FirmwareVersion` |
| `c2pa` | AI signatures | `Creator`, `Generator`, `Model` |
| `jumbf` | JUMBF container | — |
| `png` | PNG text chunks | — |

---

## 🧪 Testing

### Run full test suite

```bash
./test.sh
```

### Example test commands

```bash
# 1. Get all metadata as JSON
curl -s -X POST -F "file=@photo.jpg" http://localhost:3000/read | jq '.'

# 2. Show only GPS coordinates
curl -s -X POST -F "file=@photo.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'

# 3. Change GPS to New York
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"GPSLatitude":"40.7812","GPSLongitude":"-73.9665"}' \
  -F "type=gps" \
  http://localhost:3000/write \
  --output nyc.jpg

# 4. Remove all metadata (AI trace cleanup)
curl -X POST -F "file=@photo.jpg" http://localhost:3000/delete --output clean.jpg

# 5. Verify metadata removal
curl -s -X POST -F "file=@clean.jpg" http://localhost:3000/read | jq '.data.tag_count'
```

---

## 📚 Documentation

### Interactive Swagger UI

```bash
./docs.sh
```

Open in your browser: http://localhost:8080

### OpenAPI Specification

File: `docs/openapi.yaml`

**View online:**
- [Swagger Editor](https://editor.swagger.io/)
- [Swagger UI](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/your-repo/exif-api-rust/main/docs/openapi.yaml)

---

## 🛠️ Installation

### Project Structure

```
exif-api-rust/
├── src/
│   ├── main.rs           # Entry point
│   └── handlers.rs       # Endpoint handlers
├── docs/
│   └── openapi.yaml      # OpenAPI documentation
├── Cargo.toml            # Dependencies
├── Dockerfile            # Docker build
├── docker-compose.yml    # Docker Compose
├── setup.sh              # Project creation
├── run.sh                # Run container
├── test.sh               # Tests
├── docs.sh               # OpenAPI documentation
└── README.md             # This file
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `RUST_LOG` | Log level | `info` |

---

## ❓ FAQ

### ❓ What file formats are supported?

All formats supported by ExifTool: JPEG, PNG, WebP, TIFF, HEIC, RAW (CR2, NEF, ARW, DNG, etc.), MP4, PDF, and many more.

### ❓ How do I remove AI signatures (C2PA)?

```bash
curl -X POST -F "file=@ai_image.jpg" -F "type=c2pa" http://localhost:3000/delete --output clean.jpg
```

### ❓ Why do some tags remain after full cleanup?

ExifTool never deletes system tags like `FileType`, `ImageWidth`, `ImageHeight`, etc., so the file stays readable. This is standard behavior.

### ❓ Can I change GPS coordinates?

Absolutely! Provide the new coordinates in `metadata` with `type=gps`:

```bash
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"GPSLatitude":"40.7812","GPSLongitude":"-73.9665"}' \
  -F "type=gps" \
  http://localhost:3000/write \
  --output new_gps.jpg
```

### ❓ How can I view all tags in a file?

```bash
curl -s -X POST -F "file=@photo.jpg" http://localhost:3000/read | jq '.data.metadata | keys'
```

### ❓ Are files stored on the server?

No. All files are processed in temporary storage and automatically deleted after each request.

---

## 📄 License

MIT License — free for commercial and non‑commercial use.

---

## 🤝 Contributing

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Open a Pull Request

---

## 📞 Contact

- 📧 Email: support@example.com
- 🐛 Bugs: [GitHub Issues](https://github.com/your-repo/exif-api-rust/issues)

---

**⭐ Star the repo if you find it useful!**

---

## 📸 Example Workflow

### Original photo (San Francisco, CA)

```bash
curl -s -X POST -F "file=@photo.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'
```

```json
{
  "GPSLatitude": "37.7749° N",
  "GPSLongitude": "122.4194° W",
  "GPSAltitude": "16 m"
}
```

### After update (New York, NY — Central Park)

```bash
curl -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"GPSLatitude":"40.7812","GPSLongitude":"-73.9665"}' \
  -F "type=gps" \
  http://localhost:3000/write \
  --output nyc.jpg

curl -s -X POST -F "file=@nyc.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'
```

```json
{
  "GPSLatitude": "40.7812° N",
  "GPSLongitude": "73.9665° W",
  "GPSAltitude": "10 m"
}
```

---
## 🙏 Acknowledgments

This project would not have been possible without the incredible work of the open-source community.

- **[ExifTool](https://exiftool.org/)** by **Phil Harvey** — the industry-standard tool for reading, writing, and editing metadata. This API relies heavily on its powerful metadata processing capabilities.

- **Photo by [Zeynep S.](https://unsplash.com/@ispywithmylittleeye?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText") on [Unsplash](https://unsplash.com)** — the beautiful test image used throughout the documentation and examples. Unsplash provides stunning freely usable photography.

- **[DeepSeek](https://deepseek.com)** — the AI assistant that helped with code architecture, debugging, and documentation throughout the development process.

- **[Rust](https://www.rust-lang.org/)** and its amazing ecosystem — for making fast, reliable, and safe systems programming a joy.

- **[Docker](https://www.docker.com/)** and the container community — for making deployment simple and reproducible.

---

**Built with ❤️ and ☕ by the open-source community.**

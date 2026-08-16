use actix_multipart::Multipart;
use actix_web::{HttpResponse};
use futures_util::stream::StreamExt as _;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::NamedTempFile;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
enum MetadataType {
    All,
    Exif,
    Iptc,
    Xmp,
    MakerNotes,
    Gps,
    C2pa,
    Jumbf,
    Png,
}

impl Default for MetadataType {
    fn default() -> Self {
        MetadataType::All
    }
}

impl From<&str> for MetadataType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "exif" => MetadataType::Exif,
            "iptc" => MetadataType::Iptc,
            "xmp" => MetadataType::Xmp,
            "makernotes" | "maker" | "mn" => MetadataType::MakerNotes,
            "gps" => MetadataType::Gps,
            "c2pa" => MetadataType::C2pa,
            "jumbf" => MetadataType::Jumbf,
            "png" => MetadataType::Png,
            "all" | "" => MetadataType::All,
            _ => MetadataType::All,
        }
    }
}

impl MetadataType {
    fn to_read_args(&self) -> Vec<String> {
        match self {
            MetadataType::All => vec!["-All".to_string()],
            MetadataType::Exif => vec!["-EXIF:All".to_string()],
            MetadataType::Iptc => vec!["-IPTC:All".to_string()],
            MetadataType::Xmp => vec!["-XMP:All".to_string()],
            MetadataType::MakerNotes => vec!["-MakerNotes:All".to_string()],
            MetadataType::Gps => vec!["-GPS:All".to_string()],
            MetadataType::C2pa => vec!["-C2PA:All".to_string()],
            MetadataType::Jumbf => vec!["-JUMBF:All".to_string()],
            MetadataType::Png => vec!["-PNG:All".to_string()],
        }
    }
    
    fn to_delete_args(&self, tags: &Option<Vec<String>>) -> Vec<String> {
        let mut args = Vec::new();
        
        if let Some(ref tag_list) = tags {
            for tag in tag_list {
                let prefixed = match self {
                    MetadataType::All => format!("-{}=", tag),
                    MetadataType::Exif => format!("-EXIF:{}=", tag),
                    MetadataType::Iptc => format!("-IPTC:{}=", tag),
                    MetadataType::Xmp => format!("-XMP:{}=", tag),
                    MetadataType::MakerNotes => format!("-MakerNotes:{}=", tag),
                    MetadataType::Gps => format!("-GPS:{}=", tag),
                    MetadataType::C2pa => format!("-C2PA:{}=", tag),
                    MetadataType::Jumbf => format!("-JUMBF:{}=", tag),
                    MetadataType::Png => format!("-PNG:{}=", tag),
                };
                args.push(prefixed);
            }
        } else {
            let all_tag = match self {
                MetadataType::All => "-All=".to_string(),
                MetadataType::Exif => "-EXIF:All=".to_string(),
                MetadataType::Iptc => "-IPTC:All=".to_string(),
                MetadataType::Xmp => "-XMP:All=".to_string(),
                MetadataType::MakerNotes => "-MakerNotes:All=".to_string(),
                MetadataType::Gps => "-GPS:All=".to_string(),
                MetadataType::C2pa => "-C2PA:All=".to_string(),
                MetadataType::Jumbf => "-JUMBF:All=".to_string(),
                MetadataType::Png => "-PNG:All=".to_string(),
            };
            args.push(all_tag);
        }
        args
    }
    
    fn to_write_prefix(&self) -> &'static str {
        match self {
            MetadataType::All => "",
            MetadataType::Exif => "EXIF:",
            MetadataType::Iptc => "IPTC:",
            MetadataType::Xmp => "XMP:",
            MetadataType::MakerNotes => "MakerNotes:",
            MetadataType::Gps => "GPS:",
            MetadataType::C2pa => "C2PA:",
            MetadataType::Jumbf => "JUMBF:",
            MetadataType::Png => "PNG:",
        }
    }
}

fn normalize_value(key: &str, value: &Value) -> Value {
    if key == "SourceFile" || key == "Directory" || key == "FileName" {
        return Value::Null;
    }
    
    match value {
        Value::String(s) => {
            if s.contains("Binary data") || s.contains("(Binary") {
                return Value::Null;
            }
            let cleaned = s.replace('"', "");
            Value::String(cleaned)
        }
        Value::Number(n) => {
            if n.is_i64() {
                Value::Number(n.clone())
            } else {
                Value::Number(n.clone())
            }
        }
        Value::Bool(b) => Value::Bool(*b),
        Value::Array(arr) => {
            if arr.is_empty() {
                Value::Null
            } else {
                let strings: Vec<String> = arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if strings.is_empty() {
                    Value::Null
                } else {
                    Value::String(strings.join(", "))
                }
            }
        }
        _ => value.clone(),
    }
}

fn normalize_metadata(metadata: &serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut cleaned = serde_json::Map::new();
    
    for (key, value) in metadata {
        let normalized = normalize_value(key, value);
        if !normalized.is_null() {
            let clean_key = if let Some(idx) = key.find(':') {
                &key[idx + 1..]
            } else {
                key.as_str()
            };
            cleaned.insert(clean_key.to_string(), normalized);
        }
    }
    
    cleaned
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "version": "1.0.0",
        "supported_metadata": [
            "all", "exif", "iptc", "xmp", "makernotes", 
            "gps", "c2pa", "jumbf", "png"
        ],
        "description": "Use 'type' parameter to filter metadata groups"
    }))
}

pub async fn read_metadata(mut payload: Multipart) -> HttpResponse {
    log::info!("📖 Read metadata request received");

    let mut file_data = None;
    let mut metadata_type = MetadataType::All;
    let mut original_filename = String::new();

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            match field.name() {
                "file" => {
                    let disp = field.content_disposition();
                    if let Some(name) = disp.get_filename() {
                        original_filename = name.to_string();
                    }
                    
                    let mut temp_file = match NamedTempFile::new() {
                        Ok(f) => f,
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(json!({
                                "success": false,
                                "error": format!("Failed to create temp file: {}", e)
                            }));
                        }
                    };
                    
                    while let Some(chunk) = field.next().await {
                        if let Ok(data) = chunk {
                            if let Err(e) = temp_file.write_all(&data) {
                                return HttpResponse::InternalServerError().json(json!({
                                    "success": false,
                                    "error": format!("Failed to write file: {}", e)
                                }));
                            }
                        }
                    }
                    file_data = Some(temp_file);
                }
                "type" => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        if let Ok(chunk_data) = chunk {
                            data.extend_from_slice(&chunk_data);
                        }
                    }
                    if let Ok(type_str) = String::from_utf8(data) {
                        metadata_type = MetadataType::from(type_str.trim());
                        log::info!("📌 Metadata type: {:?}", metadata_type);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(file) = file_data {
        let file_path = file.path().to_str().unwrap();

        let mut cmd = Command::new("exiftool");
        cmd.arg("-j");
        
        for arg in metadata_type.to_read_args() {
            cmd.arg(&arg);
        }
        
        cmd.arg(file_path);

        log::info!("🔧 Running: {:?}", cmd);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let exif_data: serde_json::Value = serde_json::from_str(&stdout)
                        .unwrap_or_else(|_| json!([{}]));
                    
                    let raw_tags = if let Some(obj) = exif_data.as_array().and_then(|arr| arr.first()) {
                        obj.as_object().unwrap_or(&serde_json::Map::new()).clone()
                    } else {
                        serde_json::Map::new()
                    };

                    let cleaned_tags = normalize_metadata(&raw_tags);

                    let filename = if original_filename.is_empty() {
                        "uploaded".to_string()
                    } else {
                        original_filename
                    };

                    return HttpResponse::Ok().json(json!({
                        "success": true,
                        "data": {
                            "filename": filename,
                            "metadata_type": format!("{:?}", metadata_type),
                            "metadata": cleaned_tags,
                            "tag_count": cleaned_tags.len()
                        }
                    }));
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    log::error!("exiftool read error: {}", error_msg);
                    return HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": format!("exiftool failed: {}", error_msg),
                        "code": "METADATA_READ_ERROR"
                    }));
                }
            }
            Err(e) => {
                log::error!("Failed to execute exiftool: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to execute exiftool: {}", e),
                    "code": "METADATA_READ_ERROR"
                }));
            }
        }
    }

    HttpResponse::BadRequest().json(json!({
        "success": false,
        "error": "No file uploaded",
        "code": "MISSING_FILE"
    }))
}

pub async fn write_metadata(mut payload: Multipart) -> HttpResponse {
    log::info!("✏️ Write metadata request received");

    let mut file_data = None;
    let mut metadata = HashMap::new();
    let mut original_filename = String::new();
    let mut metadata_type = MetadataType::All;

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            match field.name() {
                "file" => {
                    let disp = field.content_disposition();
                    if let Some(name) = disp.get_filename() {
                        original_filename = name.to_string();
                    }
                    
                    let mut temp_file = NamedTempFile::new().unwrap();
                    while let Some(chunk) = field.next().await {
                        if let Ok(data) = chunk {
                            temp_file.write_all(&data).unwrap();
                        }
                    }
                    file_data = Some(temp_file);
                }
                "metadata" => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        if let Ok(chunk_data) = chunk {
                            data.extend_from_slice(&chunk_data);
                        }
                    }
                    if let Ok(json_str) = String::from_utf8(data) {
                        if let Ok(parsed) = serde_json::from_str::<HashMap<String, String>>(&json_str) {
                            metadata = parsed;
                            log::info!("📝 Received {} metadata tags to write", metadata.len());
                        }
                    }
                }
                "type" => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        if let Ok(chunk_data) = chunk {
                            data.extend_from_slice(&chunk_data);
                        }
                    }
                    if let Ok(type_str) = String::from_utf8(data) {
                        metadata_type = MetadataType::from(type_str.trim());
                        log::info!("📌 Writing metadata type: {:?}", metadata_type);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(file) = file_data {
        let file_path = file.path().to_str().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        // КРИТИЧНО: удаляем выходной файл, если он существует (exiftool не перезаписывает)
        if std::path::Path::new(output_path).exists() {
            let _ = std::fs::remove_file(output_path);
            log::debug!("Removed existing output file: {}", output_path);
        }

        let mut cmd = Command::new("exiftool");
        cmd.arg("-overwrite_original");
        
        let prefix = metadata_type.to_write_prefix();
        
        for (tag_name, tag_value) in &metadata {
            let tag_arg = if prefix.is_empty() {
                format!("-{}={}", tag_name, tag_value)
            } else {
                format!("-{}{}={}", prefix, tag_name, tag_value)
            };
            cmd.arg(&tag_arg);
            log::debug!("  {}", tag_arg);
        }
        
        cmd.arg(file_path);
        cmd.arg("-o");
        cmd.arg(output_path);

        log::info!("🔧 Running: {:?}", cmd);

        match cmd.output() {
            Ok(output) => {
                // ВСЕГДА проверяем статус exiftool!
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    log::error!("exiftool write error: {}", error_msg);
                    
                    // Возвращаем ошибку, а не битый файл
                    return HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": format!("exiftool failed: {}", error_msg),
                        "code": "METADATA_WRITE_ERROR"
                    }));
                }
                
                // Проверяем, что выходной файл существует и не пустой
                match std::fs::metadata(output_path) {
                    Ok(meta) => {
                        if meta.len() < 1000 {
                            log::warn!("Output file is too small: {} bytes", meta.len());
                            return HttpResponse::InternalServerError().json(json!({
                                "success": false,
                                "error": "Output file is too small, likely an error occurred",
                                "code": "METADATA_WRITE_ERROR"
                            }));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read output file metadata: {}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": format!("Failed to read output file: {}", e),
                            "code": "METADATA_WRITE_ERROR"
                        }));
                    }
                }
                
                let modified_data = match std::fs::read(output_path) {
                    Ok(data) => data,
                    Err(e) => {
                        log::error!("Failed to read output file: {}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": format!("Failed to read output file: {}", e),
                            "code": "METADATA_WRITE_ERROR"
                        }));
                    }
                };
                
                let filename = if original_filename.is_empty() {
                    "modified.jpg".to_string()
                } else {
                    format!("modified_{}", original_filename)
                };
                
                log::info!("✅ Successfully wrote {} tags, file size: {} bytes", metadata.len(), modified_data.len());
                
                HttpResponse::Ok()
                    .content_type("application/octet-stream")
                    .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                    .body(modified_data)
            }
            Err(e) => {
                log::error!("Failed to execute exiftool: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to execute exiftool: {}", e),
                    "code": "METADATA_WRITE_ERROR"
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Missing file or metadata",
            "code": "MISSING_DATA"
        }))
    }
}

pub async fn delete_metadata(mut payload: Multipart) -> HttpResponse {
    log::info!("🗑️ Delete metadata request received");

    let mut file_data = None;
    let mut tags_to_delete: Option<Vec<String>> = None;
    let mut original_filename = String::new();
    let mut metadata_type = MetadataType::All;

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            match field.name() {
                "file" => {
                    let disp = field.content_disposition();
                    if let Some(name) = disp.get_filename() {
                        original_filename = name.to_string();
                    }
                    
                    let mut temp_file = NamedTempFile::new().unwrap();
                    while let Some(chunk) = field.next().await {
                        if let Ok(data) = chunk {
                            temp_file.write_all(&data).unwrap();
                        }
                    }
                    file_data = Some(temp_file);
                }
                "tags" => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        if let Ok(chunk_data) = chunk {
                            data.extend_from_slice(&chunk_data);
                        }
                    }
                    if let Ok(json_str) = String::from_utf8(data) {
                        if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&json_str) {
                            tags_to_delete = Some(parsed);
                            log::info!("📝 Deleting {} specific tags", tags_to_delete.as_ref().unwrap().len());
                        }
                    }
                }
                "type" => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        if let Ok(chunk_data) = chunk {
                            data.extend_from_slice(&chunk_data);
                        }
                    }
                    if let Ok(type_str) = String::from_utf8(data) {
                        metadata_type = MetadataType::from(type_str.trim());
                        log::info!("📌 Deleting metadata type: {:?}", metadata_type);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(file) = file_data {
        let file_path = file.path().to_str().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        // КРИТИЧНО: удаляем выходной файл, если он существует
        if std::path::Path::new(output_path).exists() {
            let _ = std::fs::remove_file(output_path);
            log::debug!("Removed existing output file: {}", output_path);
        }

        let mut cmd = Command::new("exiftool");
        cmd.arg("-overwrite_original");
        
        for arg in metadata_type.to_delete_args(&tags_to_delete) {
            cmd.arg(arg);
        }
        
        cmd.arg(file_path);
        cmd.arg("-o");
        cmd.arg(output_path);

        log::info!("🔧 Running: {:?}", cmd);

        match cmd.output() {
            Ok(output) => {
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    log::error!("exiftool delete error: {}", error_msg);
                    return HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": format!("exiftool failed: {}", error_msg),
                        "code": "METADATA_DELETE_ERROR"
                    }));
                }
                
                match std::fs::metadata(output_path) {
                    Ok(meta) => {
                        if meta.len() < 1000 {
                            log::warn!("Output file is too small: {} bytes", meta.len());
                            return HttpResponse::InternalServerError().json(json!({
                                "success": false,
                                "error": "Output file is too small, likely an error occurred",
                                "code": "METADATA_DELETE_ERROR"
                            }));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read output file metadata: {}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": format!("Failed to read output file: {}", e),
                            "code": "METADATA_DELETE_ERROR"
                        }));
                    }
                }
                
                let modified_data = std::fs::read(output_path).unwrap_or_default();
                let filename = if original_filename.is_empty() {
                    "modified.jpg".to_string()
                } else {
                    format!("modified_{}", original_filename)
                };
                
                log::info!("✅ Successfully deleted tags, file size: {} bytes", modified_data.len());
                
                HttpResponse::Ok()
                    .content_type("application/octet-stream")
                    .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                    .body(modified_data)
            }
            Err(e) => {
                log::error!("Failed to execute exiftool: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to execute exiftool: {}", e),
                    "code": "METADATA_DELETE_ERROR"
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "No file uploaded",
            "code": "MISSING_FILE"
        }))
    }
}

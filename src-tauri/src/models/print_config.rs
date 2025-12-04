use serde::{Deserialize, Serialize};

/// Page orientation
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Supported paper format
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PaperFormat {
    A4,
    A3,
    Letter,
}

impl PaperFormat {
    /// Returns dimensions in mm (width, height) in portrait mode
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PaperFormat::A4 => (210.0, 297.0),
            PaperFormat::A3 => (297.0, 420.0),
            PaperFormat::Letter => (215.9, 279.4),
        }
    }
}

/// Print configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrintConfig {
    pub format: PaperFormat,
    pub orientation: Orientation,
    pub margins_mm: f32,
    pub scale: f32,
}

/// Complete print request (kept for compatibility)
#[derive(Debug, Deserialize)]
pub struct PrintRequest {
    pub config: PrintConfig,
    pub html_content: String,
    pub printer_id: Option<String>, // None = default printer
}

/// Print request with multiple pages (each page is treated individually)
#[derive(Debug, Deserialize)]
pub struct PrintRequestPages {
    pub config: PrintConfig,
    pub pages: Vec<String>, // Array of HTML contents, one for each page
    pub printer_id: Option<String>, // None = default printer
}

/// Information about an available printer
#[derive(Debug, Serialize, Clone)]
pub struct PrinterInfo {
    pub id: String,           // Technical name (e.g.: "Canon_G3000")
    pub display_name: String, // Friendly name
    pub is_default: bool,
}

/// Result of a print operation
#[derive(Debug, Serialize)]
pub struct PrintResult {
    pub success: bool,
    pub message: String,
    pub job_id: Option<String>,
}

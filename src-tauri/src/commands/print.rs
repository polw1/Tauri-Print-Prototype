use std::process::Command;
use std::fs;
use std::path::PathBuf;
use crate::models::print_config::{PrinterInfo, PrintRequest, PrintRequestPages, PrintResult, PrintConfig, Orientation};
use headless_chrome::Browser;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Lists all available printers on the system
#[tauri::command]
pub async fn get_printers() -> Result<Vec<PrinterInfo>, String> {
    #[cfg(target_os = "linux")]
    {
        get_printers_linux().await
    }
    
    #[cfg(target_os = "windows")]
    {
        get_printers_windows().await
    }
    
    #[cfg(target_os = "macos")]
    {
        get_printers_macos().await
    }
}

#[cfg(target_os = "linux")]
async fn get_printers_linux() -> Result<Vec<PrinterInfo>, String> {
    // Execute lpstat -p -d to list printers
    let output = Command::new("lpstat")
        .args(&["-p", "-d"])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}. Check if CUPS is installed.", e))?;

    if !output.status.success() {
        return Err(format!(
            "lpstat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut printers = Vec::new();
    let mut default_printer: Option<String> = None;

    for line in stdout.lines() {
        if line.starts_with("printer ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let printer_id = parts[1].to_string();
                printers.push(PrinterInfo {
                    id: printer_id.clone(),
                    display_name: printer_id.replace('_', " "),
                    is_default: false,
                });
            }
        } else if line.contains("default destination:") {
            if let Some(name) = line.split(':').nth(1) {
                default_printer = Some(name.trim().to_string());
            }
        }
    }

    if let Some(default) = default_printer {
        for printer in &mut printers {
            if printer.id == default {
                printer.is_default = true;
                break;
            }
        }
    }

    if printers.iter().all(|p| !p.is_default) && !printers.is_empty() {
        printers[0].is_default = true;
    }

    Ok(printers)
}

#[cfg(target_os = "windows")]
async fn get_printers_windows() -> Result<Vec<PrinterInfo>, String> {
    // Use PowerShell to list printers
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-Printer | Select-Object Name, Default | ConvertTo-Json"
        ])
        .output()
        .map_err(|e| format!("Error executing PowerShell: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "PowerShell failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse JSON (simplified - use serde_json in production)
    let mut printers = Vec::new();
    
    // Simple fallback if JSON parsing fails
    if stdout.contains("Name") {
        // Try to extract printer names from output
        for line in stdout.lines() {
            if line.contains("\"Name\"") {
                if let Some(name_part) = line.split(':').nth(1) {
                    let name = name_part
                        .trim()
                        .trim_matches(',')
                        .trim_matches('"')
                        .to_string();
                    if !name.is_empty() {
                        printers.push(PrinterInfo {
                            id: name.clone(),
                            display_name: name,
                            is_default: false,
                        });
                    }
                }
            }
        }
    }

    // Mark first as default if none was marked
    if !printers.is_empty() {
        printers[0].is_default = true;
    }

    Ok(printers)
}

#[cfg(target_os = "macos")]
async fn get_printers_macos() -> Result<Vec<PrinterInfo>, String> {
    let output = Command::new("lpstat")
        .args(&["-p", "-d"])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "lpstat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut printers = Vec::new();
    let mut default_printer: Option<String> = None;

    for line in stdout.lines() {
        if line.starts_with("printer ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let printer_id = parts[1].to_string();
                printers.push(PrinterInfo {
                    id: printer_id.clone(),
                    display_name: printer_id.replace('_', " "),
                    is_default: false,
                });
            }
        } else if line.contains("default destination:") {
            if let Some(name) = line.split(':').nth(1) {
                default_printer = Some(name.trim().to_string());
            }
        }
    }

    if let Some(default) = default_printer {
        for printer in &mut printers {
            if printer.id == default {
                printer.is_default = true;
                break;
            }
        }
    }

    if printers.iter().all(|p| !p.is_default) && !printers.is_empty() {
        printers[0].is_default = true;
    }

    Ok(printers)
}

/// Converts HTML to PDF using Chrome headless (no external dependencies!)
fn html_to_pdf(html_content: &str, request: &PrintRequest) -> Result<PathBuf, String> {
    // Create temporary directory
    let temp_dir = std::env::temp_dir();
    let job_id = uuid::Uuid::new_v4().to_string();
    let pdf_path = temp_dir.join(format!("print-{}.pdf", job_id));

    // Prepare full HTML with inline CSS
    let full_html = prepare_full_html(html_content, request);

    // Start Chrome headless
    let browser = Browser::default()
        .map_err(|e| format!("Error starting Chrome headless: {}. Is Chrome/Chromium installed?", e))?;

    let tab = browser.new_tab()
        .map_err(|e| format!("Error creating new tab: {}", e))?;

    // Convert HTML to data URI to avoid file:// issues on Windows
    let html_base64 = STANDARD.encode(full_html.as_bytes());
    let data_uri = format!("data:text/html;base64,{}", html_base64);

    // Navigate to HTML
    tab.navigate_to(&data_uri)
        .map_err(|e| format!("Error loading HTML: {}", e))?;

    // Wait for complete loading
    tab.wait_until_navigated()
        .map_err(|e| format!("Error waiting for navigation: {}", e))?;

    // Wait a bit more to ensure CSS/fonts are loaded
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Configure PDF options
    let (width, height) = request.config.format.dimensions_mm();
    let (page_width, page_height) = match request.config.orientation {
        Orientation::Portrait => (width, height),
        Orientation::Landscape => (height, width),
    };

    // Convert mm to inches (Chrome uses inches)
    let width_inches = (page_width / 25.4) as f64;
    let height_inches = (page_height / 25.4) as f64;

    let landscape = matches!(request.config.orientation, Orientation::Landscape);

    // Create PDF options
    use headless_chrome::types::PrintToPdfOptions;
    
    let mut pdf_options = PrintToPdfOptions::default();
    pdf_options.landscape = Some(landscape);
    pdf_options.display_header_footer = Some(false); // ← NO HEADERS/FOOTERS!
    pdf_options.print_background = Some(true);
    pdf_options.scale = Some(request.config.scale as f64);
    pdf_options.paper_width = Some(width_inches);
    pdf_options.paper_height = Some(height_inches);
    // ← REMOVED: Chrome margins (we use CSS padding in HTML instead)
    pdf_options.margin_top = Some(0.0);
    pdf_options.margin_bottom = Some(0.0);
    pdf_options.margin_left = Some(0.0);
    pdf_options.margin_right = Some(0.0);
    pdf_options.prefer_css_page_size = Some(true);

    // Generate PDF
    let pdf_data = tab.print_to_pdf(Some(pdf_options))
        .map_err(|e| format!("Error generating PDF: {}", e))?;

    // Save PDF
    fs::write(&pdf_path, &pdf_data)
        .map_err(|e| format!("Error saving PDF: {}", e))?;

    Ok(pdf_path)
}

/// Prepares full HTML with inline CSS
fn prepare_full_html(content: &str, request: &PrintRequest) -> String {
    let (width, height) = request.config.format.dimensions_mm();
    let (page_width, page_height) = match request.config.orientation {
        Orientation::Portrait => (width, height),
        Orientation::Landscape => (height, width),
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        @page {{
            size: {}mm {}mm;
            margin: 0;
        }}
        
        body {{
            width: {}mm;
            margin: 0;
            padding: 0;
            background: white;
        }}
        
        .print-page {{
            width: {}mm;
            min-height: {}mm;
            padding: {}mm;
            background: white;
            page-break-after: always;
            font-family: 'Times New Roman', serif;
            font-size: 12pt;
            line-height: 1.5;
        }}
        
        .print-page:last-child {{
            page-break-after: avoid;
        }}
        
        h1, h2, h3, h4, h5, h6 {{
            margin-bottom: 0.5em;
        }}
        
        p {{
            margin-bottom: 0.5em;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        page_width,
        page_height,
        page_width,
        page_width,
        page_height,
        request.config.margins_mm,
        content
    )
}

/// Generates PDF with multiple pages using Chrome headless (no external dependencies!)
/// Each page is treated individually but generated in a single PDF
fn generate_multi_page_pdf(pages: &[String], config: &PrintConfig) -> Result<Vec<u8>, String> {
    let (width, height) = config.format.dimensions_mm();
    let (page_width, page_height) = match config.orientation {
        Orientation::Portrait => (width, height),
        Orientation::Landscape => (height, width),
    };

    // Create HTML with multiple pages, each isolated with page-break
    let mut pages_html = String::new();
    for (idx, page_content) in pages.iter().enumerate() {
        let page_break = if idx < pages.len() - 1 { "page-break-after: always;" } else { "" };
        pages_html.push_str(&format!(
            r#"<div class="print-page" style="
                width: {}mm;
                height: {}mm;
                padding: {}mm;
                background: white;
                font-family: 'Times New Roman', serif;
                font-size: 12pt;
                line-height: 1.5;
                box-sizing: border-box;
                overflow: hidden;
                {}
            ">{}</div>"#,
            page_width,
            page_height,
            config.margins_mm,
            page_break,
            page_content
        ));
    }

    let full_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        @page {{
            size: {}mm {}mm;
            margin: 0;
        }}
        
        body {{
            margin: 0;
            padding: 0;
            background: white;
        }}
        
        .print-page {{
            page-break-inside: avoid;
        }}
        
        h1, h2, h3, h4, h5, h6 {{
            margin-bottom: 0.5em;
        }}
        
        p {{
            margin-bottom: 0.5em;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        page_width,
        page_height,
        pages_html
    );

    // Start Chrome headless
    let browser = Browser::default()
        .map_err(|e| format!("Error starting Chrome headless: {}. Is Chrome/Chromium installed?", e))?;

    let tab = browser.new_tab()
        .map_err(|e| format!("Error creating new tab: {}", e))?;

    // Convert HTML to data URI
    let html_base64 = STANDARD.encode(full_html.as_bytes());
    let data_uri = format!("data:text/html;base64,{}", html_base64);

    // Navigate to HTML
    tab.navigate_to(&data_uri)
        .map_err(|e| format!("Error loading HTML: {}", e))?;

    // Wait for complete loading
    tab.wait_until_navigated()
        .map_err(|e| format!("Error waiting for navigation: {}", e))?;

    // Wait a bit more to ensure CSS/fonts are loaded
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Convert mm to inches (Chrome uses inches)
    let width_inches = (page_width / 25.4) as f64;
    let height_inches = (page_height / 25.4) as f64;

    let landscape = matches!(config.orientation, Orientation::Landscape);

    // Create PDF options
    use headless_chrome::types::PrintToPdfOptions;
    
    let mut pdf_options = PrintToPdfOptions::default();
    pdf_options.landscape = Some(landscape);
    pdf_options.display_header_footer = Some(false);
    pdf_options.print_background = Some(true);
    pdf_options.scale = Some(config.scale as f64);
    pdf_options.paper_width = Some(width_inches);
    pdf_options.paper_height = Some(height_inches);
    pdf_options.margin_top = Some(0.0);
    pdf_options.margin_bottom = Some(0.0);
    pdf_options.margin_left = Some(0.0);
    pdf_options.margin_right = Some(0.0);
    pdf_options.prefer_css_page_size = Some(true);

    // Generate PDF
    let pdf_data = tab.print_to_pdf(Some(pdf_options))
        .map_err(|e| format!("Error generating PDF: {}", e))?;

    Ok(pdf_data)
}

/// Generates final PDF with all pages (each page treated individually)
fn generate_merged_pdf(pages: &[String], config: &PrintConfig) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let job_id = uuid::Uuid::new_v4().to_string();
    let pdf_path = temp_dir.join(format!("print-merged-{}.pdf", job_id));

    // Generate PDF with all pages using Chrome headless
    let pdf_data = generate_multi_page_pdf(pages, config)?;

    // Save final PDF
    fs::write(&pdf_path, &pdf_data)
        .map_err(|e| format!("Error saving PDF: {}", e))?;

    Ok(pdf_path)
}

/// Saves document as PDF directly to specified path
#[tauri::command]
pub async fn save_pdf_to_path(request: PrintRequest, destination_path: String) -> Result<String, String> {
    // 1. Create temporary PDF
    let temp_pdf_path = html_to_pdf(&request.html_content, &request)?;

    // 2. Move temporary file to destination
    fs::copy(&temp_pdf_path, &destination_path)
        .map_err(|e| format!("Error saving PDF: {}", e))?;

    // 3. Remove temporary
    let _ = fs::remove_file(&temp_pdf_path);

    // 4. Return final path
    Ok(destination_path)
}

/// Saves document with multiple pages as PDF (merge in backend)
#[tauri::command]
pub async fn save_pdf_pages_to_path(request: PrintRequestPages, destination_path: String) -> Result<String, String> {
    // 1. Generate merged PDF
    let temp_pdf_path = generate_merged_pdf(&request.pages, &request.config)?;

    // 2. Move temporary file to destination
    fs::copy(&temp_pdf_path, &destination_path)
        .map_err(|e| format!("Error saving PDF: {}", e))?;

    // 3. Remove temporary
    let _ = fs::remove_file(&temp_pdf_path);

    // 4. Return final path
    Ok(destination_path)
}

/// Prints document using CUPS (Linux)
#[tauri::command]
pub async fn print_document(request: PrintRequest) -> Result<PrintResult, String> {
    // 1. Convert HTML to PDF
    let pdf_path = html_to_pdf(&request.html_content, &request)?;

    // 2. Determine target printer
    let printer = if let Some(printer_id) = &request.printer_id {
        printer_id.clone()
    } else {
        // Get default printer
        let printers = get_printers().await?;
        printers
            .iter()
            .find(|p| p.is_default)
            .map(|p| p.id.clone())
            .ok_or_else(|| "No default printer found".to_string())?
    };

    // 3. Send to printer (OS dependent)
    #[cfg(target_os = "linux")]
    let result = send_to_printer_linux(&pdf_path, &printer).await;
    
    #[cfg(target_os = "windows")]
    let result = send_to_printer_windows(&pdf_path, &printer).await;
    
    #[cfg(target_os = "macos")]
    let result = send_to_printer_macos(&pdf_path, &printer).await;

    // 4. Clean up temporary PDF
    let _ = fs::remove_file(&pdf_path);

    result
}

/// Prints document with multiple pages (merge in backend)
#[tauri::command]
pub async fn print_document_pages(request: PrintRequestPages) -> Result<PrintResult, String> {
    // 1. Generate merged PDF from all pages
    let pdf_path = generate_merged_pdf(&request.pages, &request.config)?;

    // 2. Determine target printer
    let printer = if let Some(printer_id) = &request.printer_id {
        printer_id.clone()
    } else {
        // Get default printer
        let printers = get_printers().await?;
        printers
            .iter()
            .find(|p| p.is_default)
            .map(|p| p.id.clone())
            .ok_or_else(|| "No default printer found".to_string())?
    };

    // 3. Send to printer (OS dependent)
    #[cfg(target_os = "linux")]
    let result = send_to_printer_linux(&pdf_path, &printer).await;
    
    #[cfg(target_os = "windows")]
    let result = send_to_printer_windows(&pdf_path, &printer).await;
    
    #[cfg(target_os = "macos")]
    let result = send_to_printer_macos(&pdf_path, &printer).await;

    // 4. Clean up temporary PDF
    let _ = fs::remove_file(&pdf_path);

    result
}

#[cfg(target_os = "linux")]
async fn send_to_printer_linux(pdf_path: &std::path::Path, printer: &str) -> Result<PrintResult, String> {
    let output = Command::new("lp")
        .args(&[
            "-d", printer,
            "-o", "fit-to-page",
        ])
        .arg(pdf_path)
        .output()
        .map_err(|e| format!("Error executing lp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(PrintResult {
            success: false,
            message: format!("Failed to send to printer: {}", stderr),
            job_id: None,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let job_id = stdout.split_whitespace().last().map(|s| s.to_string());

    Ok(PrintResult {
        success: true,
        message: format!("Document sent to printer: {}", printer),
        job_id,
    })
}

#[cfg(target_os = "windows")]
async fn send_to_printer_windows(pdf_path: &std::path::Path, printer: &str) -> Result<PrintResult, String> {
    // On Windows, use PowerShell to print
    let pdf_path_str = pdf_path.to_string_lossy();
    
    let ps_command = format!(
        "Start-Process -FilePath '{}' -Verb Print -ArgumentList '/d:\"{}\"'",
        pdf_path_str, printer
    );

    let output = Command::new("powershell")
        .args(&["-Command", &ps_command])
        .output()
        .map_err(|e| format!("Error executing PowerShell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(PrintResult {
            success: false,
            message: format!("Failed to send to printer: {}", stderr),
            job_id: None,
        });
    }

    Ok(PrintResult {
        success: true,
        message: format!("Document sent to printer: {}", printer),
        job_id: Some("windows-print-job".to_string()),
    })
}

#[cfg(target_os = "macos")]
async fn send_to_printer_macos(pdf_path: &std::path::Path, printer: &str) -> Result<PrintResult, String> {
    let output = Command::new("lpr")
        .args(&["-P", printer])
        .arg(pdf_path)
        .output()
        .map_err(|e| format!("Error executing lpr: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(PrintResult {
            success: false,
            message: format!("Failed to send to printer: {}", stderr),
            job_id: None,
        });
    }

    Ok(PrintResult {
        success: true,
        message: format!("Document sent to printer: {}", printer),
        job_id: None,
    })
}

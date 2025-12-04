mod commands;
mod models;

use commands::print::{get_printers, print_document, print_document_pages, save_pdf_to_path, save_pdf_pages_to_path};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_printers,
            print_document,
            print_document_pages,
            save_pdf_to_path,
            save_pdf_pages_to_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

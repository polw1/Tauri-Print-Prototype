export type Orientation = 'portrait' | 'landscape';
export type PaperFormat = 'A4' | 'A3' | 'Letter';

export interface PageData {
  id: string;
  content: string; // HTML content
}

export interface PrintSettings {
  format: PaperFormat;
  orientation: Orientation;
  margins: number; 
}

export const PAPER_DIMENSIONS: Record<PaperFormat, { width: number; height: number }> = {
  A4: { width: 210, height: 297 },      // mm
  A3: { width: 297, height: 420 },      // mm
  Letter: { width: 215.9, height: 279.4 } // mm
};


export interface PrinterInfo {
  id: string;
  display_name: string;
  is_default: boolean;
}

export interface TauriPrintConfig {
  format: PaperFormat;
  orientation: Orientation;
  margins_mm: number;
  scale: number;
}

export interface TauriPrintRequest {
  config: TauriPrintConfig;
  html_content: string;
  printer_id?: string; 
}

export interface TauriPrintRequestPages {
  config: TauriPrintConfig;
  pages: string[]; 
  printer_id?: string; 
}

export interface PrintResult {
  success: boolean;
  message: string;
  job_id?: string;
}


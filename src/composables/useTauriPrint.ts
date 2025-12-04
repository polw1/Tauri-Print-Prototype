import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import type { 
  TauriPrintRequestPages, 
  PrintSettings,
  PageData,
  PAPER_DIMENSIONS 
} from '../types/print';

export function useTauriPrint() {
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Save document as PDF (each page treated individually, merged in backend)
   */
  const saveAsPDF = async (
    pages: PageData[],
    settings: PrintSettings,
    _paperDimensions: typeof PAPER_DIMENSIONS
  ): Promise<{ success: boolean; message: string; path?: string }> => {
    isLoading.value = true;
    error.value = null;

    try {
      // Open save dialog FIRST
      const savePath = await save({
        defaultPath: 'document.pdf',
        filters: [{
          name: 'PDF',
          extensions: ['pdf']
        }]
      });

      if (!savePath) {
        // User cancelled
        isLoading.value = false;
        return {
          success: false,
          message: 'Operation cancelled by user'
        };
      }

      // Extract content from each page
      const pagesContent = pages.map(page => page.content);

      // Prepare request with separate pages
      const request: TauriPrintRequestPages = {
        config: {
          format: settings.format,
          orientation: settings.orientation,
          margins_mm: settings.margins,
          scale: 1.0
        },
        pages: pagesContent
      };

      // Save PDF with merge in backend
      const finalPath = await invoke<string>('save_pdf_pages_to_path', { 
        request,
        destinationPath: savePath 
      });

      return {
        success: true,
        message: 'PDF saved successfully',
        path: finalPath
      };
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      error.value = errorMsg;
      throw new Error(errorMsg);
    } finally {
      isLoading.value = false;
    }
  };

  return {
    isLoading,
    error,
    saveAsPDF
  };
}

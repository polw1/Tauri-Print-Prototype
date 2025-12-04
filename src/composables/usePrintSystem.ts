import { ref, computed, watchEffect } from 'vue';
import { type PageData, type PrintSettings, PAPER_DIMENSIONS } from '../types/print';

export function usePrintSystem() {
  // Document State
  const pages = ref<PageData[]>([
    { id: crypto.randomUUID(), content: '<h1>Hello World</h1><p>Start editing your document here...</p>' }
  ]);

  // Print Settings
  const settings = ref<PrintSettings>({
    format: 'A4',
    orientation: 'portrait',
    margins: 20
  });

  // Page Actions
  const addPage = () => {
    pages.value.push({
      id: crypto.randomUUID(),
      content: ''
    });
  };

  const removePage = (id: string) => {
    if (pages.value.length > 1) {
      pages.value = pages.value.filter(p => p.id !== id);
    }
  };

  const updatePageContent = (id: string, newContent: string) => {
    const page = pages.value.find(p => p.id === id);
    if (page) {
      page.content = newContent;
    }
  };

  // Style Calculations with Auto Zoom (CSS Variables)
  const pageStyles = computed(() => {
    const dim = PAPER_DIMENSIONS[settings.value.format];
    const isPortrait = settings.value.orientation === 'portrait';

    const widthMm = isPortrait ? dim.width : dim.height;
    const heightMm = isPortrait ? dim.height : dim.width;

    // Calculate auto zoom to fit viewport
    // Assuming viewport has ~900px available width and ~600px available height
    const availableWidth = 900; // px
    const availableHeight = 600; // px
    const mmToPx = 3.78; // 1mm ≈ 3.78px at 96dpi

    const pageWidthPx = widthMm * mmToPx;
    const pageHeightPx = heightMm * mmToPx;

    // Calculate scale to fit both width and height, with some padding
    const scaleX = (availableWidth - 80) / pageWidthPx; // 80px padding
    const scaleY = (availableHeight - 80) / pageHeightPx; // 80px padding
    const autoScale = Math.min(scaleX, scaleY, 1.0); // Never scale up, only down

    return {
      width: `${widthMm}mm`,
      height: `${heightMm}mm`,
      padding: `${settings.value.margins}mm`,
      // CSS variables for internal use if needed
      '--page-width': `${widthMm}mm`,
      '--page-height': `${heightMm}mm`,
      '--auto-scale': autoScale.toString(),
    };
  });

  // @page CSS Injection Logic (CSS Force)
  watchEffect(() => {
    const styleId = 'print-force-style';
    let styleEl = document.getElementById(styleId) as HTMLStyleElement;

    if (!styleEl) {
      styleEl = document.createElement('style');
      styleEl.id = styleId;
      document.head.appendChild(styleEl);
    }

    const { format, orientation } = settings.value;
    styleEl.innerHTML = `
      @page {
        size: ${format} ${orientation};
        margin: 0 !important;
      }
      
      /* Try to hide browser-generated headers (Chrome/Webkit) */
      @media print {
        body {
          margin: 0;
          -webkit-print-color-adjust: exact;
          print-color-adjust: exact;
        }
      }
    `;
  });

  return {
    pages,
    settings,
    pageStyles,
    addPage,
    removePage,
    updatePageContent
  };
}

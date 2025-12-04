<script setup lang="ts">
import { ref } from 'vue';
import { usePrintSystem } from './composables/usePrintSystem';
import { useTauriPrint } from './composables/useTauriPrint';
import { PAPER_DIMENSIONS } from './types/print';
import PrintToolbar from './components/PrintToolbar.vue';
import PrintPage from './components/PrintPage.vue';
import PrinterSelectModal from './components/PrinterSelectModal.vue';

const { 
  pages, 
  settings, 
  pageStyles, 
  addPage, 
  removePage, 
  updatePageContent
} = usePrintSystem();

const {
  isLoading,
  error,
  saveAsPDF
} = useTauriPrint();

const showModal = ref(false);

// Save as PDF
const handleSavePDF = async () => {
  showModal.value = true;
  
  try {
    const result = await saveAsPDF(
      pages.value,
      settings.value,
      PAPER_DIMENSIONS
    );

    showModal.value = false;

    if (result.success) {
      alert(`✅ ${result.message}${result.path ? `\nSaved to: ${result.path}` : ''}`);
    } else {
      // User cancelled - no alert needed
      if (result.message !== 'Operation cancelled by user') {
        alert(`ℹ️ ${result.message}`);
      }
    }
  } catch (e) {
    showModal.value = false;
    alert(`❌ Error saving PDF: ${e instanceof Error ? e.message : String(e)}`);
  }
};
</script>

<template>
  <div class="app-container">
    <PrintToolbar 
      :settings="settings"
      @update:format="settings.format = $event"
      @update:orientation="settings.orientation = $event"
      @add-page="addPage"
      @save-pdf="handleSavePDF"
    />

    <main class="workspace">
      <div class="documents-scroll" :style="{ transform: `scale(${pageStyles['--auto-scale']})`, transformOrigin: 'top center' }">
        <PrintPage
          v-for="(page, index) in pages"
          :key="page.id"
          :id="page.id"
          :page-number="index + 1"
          :model-value="page.content"
          :styles="pageStyles"
          @update:model-value="updatePageContent(page.id, $event)"
          @remove="removePage"
        />
        
        <!-- Add page button at the end of document -->
        <div class="workspace-actions no-print">
          <button @click="addPage" class="add-page-big">+ Add Page</button>
        </div>
      </div>
    </main>

    <!-- Loading Modal -->
    <PrinterSelectModal
      :show="showModal"
      :is-loading="isLoading"
      :error="error"
      @close="showModal = false"
    />
  </div>
</template>

<style>
/* Reset and Base */
:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  background-color: #eef2f5; /* Workspace gray background */
  color: #213547;
}

body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100vh;
  overflow: hidden; /* Internal scroll in workspace */
}

/* App Layout */
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.workspace {
  flex: 1;
  overflow: auto;
  display: flex;
  justify-content: center;
  padding: 40px 20px;
}

.documents-scroll {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.workspace-actions {
  margin-top: 1rem;
  margin-bottom: 20px;
}

.add-page-big {
  padding: 12px 24px;
  background: transparent;
  border: 2px dashed #cbd5e0;
  border-radius: 8px;
  color: #718096;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.add-page-big:hover {
  border-color: #3b82f6;
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

/* =========================================
   PRINT CSS (CRITICAL)
   ========================================= */
@media print {
  /* 1. Hide App UI */
  .no-print, 
  .toolbar, 
  .workspace-actions, 
  .page-meta {
    display: none !important;
  }

  /* 2. Reset Layout */
  body, .app-container, .workspace, .documents-scroll {
    display: block;
    height: auto;
    overflow: visible;
    background: white;
    margin: 0;
    padding: 0;
    transform: none !important;
  }

  /* 3. Configure Page (@page) */
  @page {
    margin: 0; /* Remove default browser margins, we control via padding in component */
    size: auto; /* Let browser respect CSS break-after */
  }
}
</style>

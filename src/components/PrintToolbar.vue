<script setup lang="ts">
import { type PrintSettings, type PaperFormat, type Orientation } from '../types/print';

defineProps<{
  settings: PrintSettings;
}>();

defineEmits<{
  (e: 'update:format', val: PaperFormat): void;
  (e: 'update:orientation', val: Orientation): void;
  (e: 'save-pdf'): void;
  (e: 'add-page'): void;
}>();
</script>

<template>
  <header class="toolbar no-print">
    <div class="brand">
      📄 PrintPro
    </div>

    <div class="controls">
      <div class="control-group">
        <label>Paper:</label>
        <select 
          :value="settings.format" 
          @change="$emit('update:format', ($event.target as HTMLSelectElement).value as PaperFormat)"
        >
          <option value="A4">A4</option>
          <option value="A3">A3</option>
          <option value="Letter">Letter</option>
        </select>
      </div>

      <div class="control-group">
        <label>Orientation:</label>
        <select 
          :value="settings.orientation" 
          @change="$emit('update:orientation', ($event.target as HTMLSelectElement).value as Orientation)"
        >
          <option value="portrait">Portrait</option>
          <option value="landscape">Landscape</option>
        </select>
      </div>
    </div>

    <div class="actions">
      <button class="btn-secondary" @click="$emit('add-page')">+ New Page</button>
      <button class="btn-primary" @click="$emit('save-pdf')" title="Save document as PDF">
        💾 Save PDF
      </button>
    </div>
  </header>
</template>

<style scoped>
.toolbar {
  height: 60px;
  background: #2c3e50;
  color: white;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.2);
  position: sticky;
  top: 0;
  z-index: 100;
}

.brand {
  font-weight: bold;
  font-size: 1.2rem;
}

.controls {
  display: flex;
  gap: 20px;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

select {
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #4a5d6e;
  background: #34495e;
  color: white;
  cursor: pointer;
}

.actions {
  display: flex;
  gap: 10px;
}

button {
  padding: 6px 12px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  font-weight: 500;
  transition: filter 0.2s;
}

button:hover {
  filter: brightness(1.1);
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-secondary {
  background: #34495e;
  color: white;
  border: 1px solid #4a5d6e;
}
</style>

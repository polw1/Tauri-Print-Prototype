<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

const props = defineProps<{
  id: string;
  modelValue: string;
  pageNumber: number;
  styles: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'remove', id: string): void;
}>();

const editorRef = ref<HTMLElement | null>(null);

// Initialize content
onMounted(() => {
  if (editorRef.value) {
    editorRef.value.innerHTML = props.modelValue;
  }
});

// Watch for external content changes (e.g.: new empty page, reset, load)
watch(() => props.modelValue, (newValue) => {
  if (editorRef.value && editorRef.value.innerHTML !== newValue) {
    // Only update if different to avoid losing cursor position
    // Note: This may still move cursor to start if prop changes externally while typing
    // But it's necessary to fix initialization/recycling bugs
    editorRef.value.innerHTML = newValue;
  }
});

const onInput = () => {
  if (editorRef.value) {
    emit('update:modelValue', editorRef.value.innerHTML);
  }
};
</script>

<template>
  <div class="print-page-wrapper">
    <!-- Page header (screen only) -->
    <div class="page-meta no-print">
      <span>Page {{ pageNumber }}</span>
      <button @click="$emit('remove', id)" class="remove-btn" title="Remove page">×</button>
    </div>

    <!-- The Paper Sheet -->
    <div 
      class="print-page" 
      :style="styles"
    >
      <div 
        ref="editorRef"
        class="page-content"
        contenteditable="true"
        @input="onInput"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.print-page-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 1rem;
  width: fit-content;
}

.page-meta {
  width: 100%;
  max-width: v-bind('styles.width');
  display: flex;
  justify-content: space-between;
  color: #666;
  font-size: 0.8rem;
  margin-bottom: 0.5rem;
  padding: 0 4px;
}

.remove-btn {
  background: transparent;
  border: none;
  color: #999;
  cursor: pointer;
  font-size: 1.2rem;
  line-height: 1;
}

.remove-btn:hover {
  color: #d32f2f;
}

/* Paper Sheet Style on Screen (Skeuomorphism) */
.print-page {
  background: white;
  box-shadow: 0 4px 15px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.05);
  transition: width 0.3s ease, height 0.3s ease;
  overflow: hidden;
  flex-shrink: 0; /* Don't shrink the page */
  /* Padding comes from dynamic style prop */
}

.page-content {
  width: 100%;
  height: 100%;
  outline: none;
  font-family: 'Times New Roman', serif; /* Document default */
  font-size: 12pt;
  line-height: 1.5;
}

/* Print Styles */
@media print {
  .no-print {
    display: none !important;
  }

  .print-page-wrapper {
    margin: 0;
    padding: 0;
    display: block; /* Remove flexbox behavior on print if needed */
  }

  .print-page {
    box-shadow: none;
    margin: 0;
    border: none;
    overflow: hidden !important; /* CRITICAL: Prevents content overflow and extra page creation */
    transform: none !important; /* Remove zoom scaling for print */
    /* Ensures each page is a physical sheet */
    break-after: page;
    page-break-after: always;
  }
}
</style>

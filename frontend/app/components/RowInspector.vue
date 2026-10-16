<script setup lang="ts">
import type { ColumnInfo } from '../../shared/database'
defineProps<{
  row: Record<string, unknown> | null
  columns: ColumnInfo[]
  rowNumber: number | null
}>()
const emit = defineEmits<{ close: [] }>()
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
onMounted(() => dialog.value?.showModal())
const copied = ref(false)
const copyError = ref('')
async function copy(row: Record<string, unknown>) {
  try {
    await navigator.clipboard.writeText(JSON.stringify(row, null, 2))
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1800)
  } catch {
    copyError.value = 'Clipboard unavailable in this browser.'
  }
}
function value(input: unknown) {
  return input === null
    ? 'NULL'
    : typeof input === 'object'
      ? JSON.stringify(input, null, 2)
      : String(input)
}
</script>
<template>
  <dialog
    ref="dialog"
    class="record-dialog"
    aria-label="Record details"
    @close="emit('close')"
    @click="$event.target === dialog && dialog?.close()"
  >
    <div class="inspector">
      <div class="inspector-heading">
        <span>Record details</span
        ><button
          class="icon-button"
          title="Close details"
          aria-label="Close details"
          autofocus
          @click="dialog?.close()"
        >
          <AppIcon name="close" :size="16" />
        </button>
      </div>
      <template v-if="row"
        ><div class="record-meta">
          <span class="tiny-dot" />Row {{ rowNumber }}<span>Read only</span>
        </div>
        <dl class="record-fields">
          <div v-for="column in columns" :key="column.name">
            <dt>{{ column.name }}<AppIcon v-if="column.primaryKey" name="key" :size="12" /></dt>
            <dd :class="{ 'null-cell': row[column.name] === null }">
              {{ value(row[column.name]) }}
            </dd>
            <span class="field-type">{{ column.dataType }}</span>
          </div>
        </dl>
        <div class="inspector-footer">
          <button class="button secondary" @click="copy(row)">
            <AppIcon :name="copied ? 'check' : 'download'" :size="15" />{{
              copied ? 'Copied' : 'Copy record as JSON'
            }}
          </button>
          <p v-if="copyError" role="status" class="muted">{{ copyError }}</p>
        </div></template
      >
      <div v-else class="inspector-empty">
        <AppIcon name="panel" :size="28" />
        <h3>A closer look</h3>
        <p>Double-click a row to explore<br />its fields and values.</p>
        <kbd>Enter</kbd><span> to inspect a focused row</span>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import type { TablePage } from '../../shared/database'
const props = defineProps<{
  data: TablePage
  sort?: string
  descending: boolean
  filter: string
  sortable?: boolean
}>()
const emit = defineEmits<{ inspect: [index: number]; sort: [column: string] }>()
const selectedCell = ref<{ row: number; column: string | null } | null>(null)
watch([() => props.data, () => props.filter], () => {
  selectedCell.value = null
})
function display(value: unknown) {
  return value === null ? 'NULL' : typeof value === 'object' ? JSON.stringify(value) : String(value)
}
function matches(row: Record<string, unknown>, filter: string) {
  return (
    !filter ||
    Object.values(row).some((value) => display(value).toLowerCase().includes(filter.toLowerCase()))
  )
}
</script>
<template>
  <div class="grid-scroll" tabindex="0" aria-label="Table data, scroll to see more columns">
    <table class="data-grid">
      <thead>
        <tr>
          <th class="row-number">#</th>
          <th
            v-for="column in data.columns"
            :key="column.name"
            :aria-sort="sort === column.name ? (descending ? 'descending' : 'ascending') : 'none'"
          >
            <button
              :title="sortable === false ? column.name : `Sort by ${column.name}`"
              :disabled="sortable === false"
              @click="emit('sort', column.name)"
            >
              <span class="column-label"
                ><AppIcon v-if="column.primaryKey" name="key" :size="12" />{{ column.name
                }}<AppIcon
                  v-if="sort === column.name"
                  name="down"
                  :size="12"
                  :class="{ flipped: !descending }" /></span
              ><span class="column-type">{{ column.dataType }}</span>
            </button>
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="(row, index) in data.rows" :key="index"
          ><tr
            v-if="matches(row, filter)"
            @dblclick="emit('inspect', index)"
            @keydown.enter.prevent="emit('inspect', index)"
          >
            <td
              class="row-number"
              :class="{
                'selected-cell': selectedCell?.row === index && selectedCell.column === null,
              }"
              tabindex="0"
              title="Double-click or press Enter for row details"
              @focus="selectedCell = { row: index, column: null }"
              @click="selectedCell = { row: index, column: null }"
            >
              {{ data.page * data.pageSize + index + 1 }}
            </td>
            <td
              v-for="column in data.columns"
              :key="column.name"
              :class="{
                'null-cell': row[column.name] === null,
                'boolean-cell': typeof row[column.name] === 'boolean',
                'selected-cell': selectedCell?.row === index && selectedCell.column === column.name,
              }"
              :title="display(row[column.name])"
              tabindex="0"
              @focus="selectedCell = { row: index, column: column.name }"
              @click="selectedCell = { row: index, column: column.name }"
            >
              <span
                v-if="typeof row[column.name] === 'boolean'"
                class="boolean-value"
                :class="{ truthy: row[column.name] }"
                ><span class="tiny-dot" />{{ row[column.name] }}</span
              ><span v-else>{{ display(row[column.name]) }}</span>
            </td>
          </tr></template
        >
      </tbody>
    </table>
    <div v-if="!data.rows.some((row) => matches(row, filter))" class="grid-empty">
      <AppIcon name="search" :size="24" />
      <h3>{{ filter ? 'No matches on this page' : 'This table is empty' }}</h3>
      <p>
        {{
          filter
            ? 'Try another filter, or clear it to see all rows.'
            : 'The table structure is available in the Structure tab.'
        }}
      </p>
    </div>
  </div>
</template>

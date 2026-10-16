<script setup lang="ts">
defineProps<{ name: string; size?: number }>()
const paths: Record<string, string> = {
  database:
    'M20 6c0 2.2-3.6 4-8 4S4 8.2 4 6s3.6-4 8-4 8 1.8 8 4ZM4 6v12c0 2.2 3.6 4 8 4s8-1.8 8-4V6M4 12c0 2.2 3.6 4 8 4s8-1.8 8-4',
  table: 'M3 4h18v16H3zM3 9h18M8 9v11M15 9v11',
  search: 'm21 21-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0',
  plus: 'M12 5v14M5 12h14',
  close: 'm6 6 12 12M6 18 18 6',
  chevron: 'm9 5 7 7-7 7',
  down: 'm6 9 6 6 6-6',
  refresh: 'M20 7v5h-5M4 17v-5h5M5.3 7a8 8 0 0 1 13.2-2L20 7M4 17l1.5 2a8 8 0 0 0 13.2-2',
  panel: 'M3 4h18v16H3zM15 4v16',
  key: 'M14 8a5 5 0 1 1-4 5L3 20H1v-3l8-8M17 7h.01',
  lock: 'M5 10h14v11H5zM8 10V6a4 4 0 0 1 8 0v4M12 14v3',
  arrow: 'M5 12h14m-5-5 5 5-5 5',
  download: 'M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4',
  disconnect: 'M12 2v10M6 5a9 9 0 1 0 12 0',
  check: 'm5 12 4 4L19 6',
  grid: 'M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z',
  sort: 'M8 4v16m-4-4 4 4 4-4M16 20V4m-4 4 4-4 4 4',
}
</script>
<template>
  <svg
    :width="size || 18"
    :height="size || 18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.55"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path :d="paths[name] || paths.database" />
  </svg>
</template>

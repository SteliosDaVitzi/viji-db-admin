<script setup lang="ts">
import settings from '@material-symbols/svg-400/outlined/settings.svg?raw'
import database from '@material-symbols/svg-400/outlined/database.svg?raw'
import table from '@material-symbols/svg-400/outlined/table_chart.svg?raw'
import search from '@material-symbols/svg-400/outlined/search.svg?raw'
import plus from '@material-symbols/svg-400/outlined/add.svg?raw'
import close from '@material-symbols/svg-400/outlined/close.svg?raw'
import chevron from '@material-symbols/svg-400/outlined/chevron_right.svg?raw'
import down from '@material-symbols/svg-400/outlined/keyboard_arrow_down.svg?raw'
import refresh from '@material-symbols/svg-400/outlined/refresh.svg?raw'
import panel from '@material-symbols/svg-400/outlined/side_navigation.svg?raw'
import key from '@material-symbols/svg-400/outlined/key.svg?raw'
import lock from '@material-symbols/svg-400/outlined/lock.svg?raw'
import arrow from '@material-symbols/svg-400/outlined/arrow_forward.svg?raw'
import download from '@material-symbols/svg-400/outlined/download.svg?raw'
import disconnect from '@material-symbols/svg-400/outlined/power_settings_new.svg?raw'
import check from '@material-symbols/svg-400/outlined/check.svg?raw'
import grid from '@material-symbols/svg-400/outlined/grid_view.svg?raw'
import sort from '@material-symbols/svg-400/outlined/sort.svg?raw'
import play from '@material-symbols/svg-400/outlined/play_arrow.svg?raw'

const props = defineProps<{ name: string; size?: number }>()
// Unmodified Google Material Symbols from the pinned package; no remote requests.
const icons: Record<string, string> = {
  settings,
  database,
  table,
  search,
  plus,
  close,
  chevron,
  down,
  refresh,
  panel,
  key,
  lock,
  arrow,
  download,
  disconnect,
  check,
  grid,
  sort,
  play,
}
const paths = computed(() =>
  [...(icons[props.name] || database).matchAll(/<path d="([^"]+)"/g)].map((match) => match[1]),
)
</script>
<template>
  <svg
    :width="size || 18"
    :height="size || 18"
    viewBox="0 -960 960 960"
    fill="currentColor"
    aria-hidden="true"
    focusable="false"
  >
    <path v-for="(path, index) in paths" :key="index" :d="path" />
  </svg>
</template>

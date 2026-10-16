<script setup lang="ts">
import type { TableInfo } from '../../shared/database'
import { layoutSchemaTables } from '../utils/schemaFlowLayout'

const props = defineProps<{
  database: string
  tables: TableInfo[]
  selectedTable: TableInfo | null
  loading: boolean
}>()
const emit = defineEmits<{
  close: []
  choose: [table: TableInfo]
}>()

const nodeWidth = 292
const columnHeight = 30
const shownColumns = 8
const schemaColors = ['#93b8f2', '#8fc9a6', '#d9b77e', '#c2a6e8', '#e6a39d', '#8ac9d0']
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const viewport = useTemplateRef<HTMLDivElement>('viewport')
const filter = ref('')
const zoom = ref(1)
const pan = ref({ x: 0, y: 0 })
const positions = ref<Record<string, { x: number; y: number }>>({})
const hoveredTable = ref<string | null>(null)
const activeTable = ref<string | null>(null)
const moving = shallowRef<
  | { kind: 'pan'; pointerId: number; x: number; y: number; originX: number; originY: number }
  | {
      kind: 'node'
      pointerId: number
      key: string
      x: number
      y: number
      originX: number
      originY: number
    }
  | null
>(null)

function tableKey(table: Pick<TableInfo, 'schema' | 'name'>) {
  return JSON.stringify([table.schema, table.name])
}
function foreignKeys(table: TableInfo) {
  return table.foreignKeys ?? []
}
function primaryKeys(table: TableInfo) {
  return table.primaryKeys ?? []
}
function isForeignKey(table: TableInfo, column: string) {
  return foreignKeys(table).some((key) => key.column === column)
}
function isRelated(table: TableInfo) {
  const key = hoveredTable.value ?? activeTable.value
  if (!key) return false
  return edges.value.some(
    (edge) =>
      (edge.sourceKey === key && edge.targetKey === tableKey(table)) ||
      (edge.targetKey === key && edge.sourceKey === tableKey(table)),
  )
}
function tableHeight(table: TableInfo) {
  const visible = Math.min(table.columns.length, shownColumns)
  const fieldsHeight = table.columns.length
    ? 10 + visible * columnHeight + (table.columns.length > shownColumns ? 30 : 0)
    : 41
  return 56 + fieldsHeight + 39
}
const sortedTables = computed(() =>
  [...props.tables].sort(
    (a, b) => a.schema.localeCompare(b.schema) || a.name.localeCompare(b.name),
  ),
)
const flowPositions = computed(() => layoutSchemaTables(sortedTables.value))
watch(
  flowPositions,
  (layout, previousLayout) => {
    positions.value = { ...layout }
    if (previousLayout) void fitView()
  },
  { immediate: true },
)
function positionOf(table: TableInfo) {
  return positions.value[tableKey(table)] ?? { x: 72, y: 72 }
}
const visibleTables = computed(() => {
  const query = filter.value.trim().toLowerCase()
  if (!query) return sortedTables.value
  return sortedTables.value.filter((table) =>
    `${table.schema}.${table.name} ${table.columns.join(' ')}`.toLowerCase().includes(query),
  )
})
const relationshipMetadataMissing = computed(
  () =>
    props.tables.length > 0 && props.tables.every((table) => !Object.hasOwn(table, 'foreignKeys')),
)
const visibleKeys = computed(() => new Set(visibleTables.value.map(tableKey)))
const relationships = computed(() =>
  visibleTables.value.flatMap((source) =>
    foreignKeys(source)
      .filter((key) =>
        visibleKeys.value.has(
          tableKey({ schema: key.referencedSchema, name: key.referencedTable }),
        ),
      )
      .map((foreignKey) => ({ source, foreignKey })),
  ),
)
const relationshipCount = computed(() =>
  props.tables.reduce(
    (count, table) =>
      count +
      foreignKeys(table).filter((key) =>
        props.tables.some(
          (target) => target.schema === key.referencedSchema && target.name === key.referencedTable,
        ),
      ).length,
    0,
  ),
)
function rowY(table: TableInfo, column: string) {
  const index = table.columns.indexOf(column)
  if (index >= shownColumns) return positionOf(table).y + 56 + 5 + shownColumns * columnHeight + 15
  const row = index < 0 ? 0 : index
  return positionOf(table).y + 56 + 5 + row * columnHeight + columnHeight / 2
}
const edges = computed(() =>
  relationships.value.flatMap(({ source: dependent, foreignKey }) => {
    const parent = props.tables.find(
      (table) =>
        table.schema === foreignKey.referencedSchema && table.name === foreignKey.referencedTable,
    )
    if (!parent) return []
    const sourcePosition = positionOf(parent)
    const targetPosition = positionOf(dependent)
    const sourceKey = tableKey(parent)
    const targetKey = tableKey(dependent)
    const sourceColumn = foreignKey.referencedColumn
    const targetColumn = foreignKey.column
    const label = `${parent.schema}.${parent.name}.${sourceColumn} \u2192 ${dependent.schema}.${dependent.name}.${targetColumn}`
    let startX: number
    let endX: number
    let startY: number
    let endY: number
    let bend: number

    if (sourceKey === targetKey) {
      startX = sourcePosition.x + nodeWidth
      endX = startX
      startY = rowY(parent, sourceColumn)
      endY = rowY(dependent, targetColumn)
      return [
        {
          id: `${sourceKey}:${foreignKey.name}:${foreignKey.column}`,
          sourceKey,
          targetKey,
          label,
          path: `M ${startX} ${startY} C ${startX + 86} ${startY - 42}, ${endX + 86} ${endY + 42}, ${endX} ${endY}`,
        },
      ]
    }

    const sourceCenterX = sourcePosition.x + nodeWidth / 2
    const targetCenterX = targetPosition.x + nodeWidth / 2
    const sourceHeight = tableHeight(parent)
    const targetHeight = tableHeight(dependent)
    const verticalDelta =
      targetPosition.y + targetHeight / 2 - (sourcePosition.y + sourceHeight / 2)
    const horizontalDelta = targetCenterX - sourceCenterX

    if (Math.abs(verticalDelta) >= Math.abs(horizontalDelta)) {
      const down = verticalDelta >= 0
      startX = sourceCenterX
      endX = targetCenterX
      startY = down ? sourcePosition.y + sourceHeight : sourcePosition.y
      endY = down ? targetPosition.y : targetPosition.y + targetHeight
      bend = Math.max(48, Math.abs(endY - startY) * 0.4)
      return [
        {
          id: `${sourceKey}:${foreignKey.name}:${foreignKey.column}`,
          sourceKey,
          targetKey,
          label,
          path: `M ${startX} ${startY} C ${startX} ${startY + (down ? bend : -bend)}, ${endX} ${endY - (down ? bend : -bend)}, ${endX} ${endY}`,
        },
      ]
    }

    const right = horizontalDelta > 0
    startX = right ? sourcePosition.x + nodeWidth : sourcePosition.x
    endX = right ? targetPosition.x : targetPosition.x + nodeWidth
    startY = rowY(parent, sourceColumn)
    endY = rowY(dependent, targetColumn)
    bend = Math.max(48, Math.abs(endX - startX) * 0.4)
    return [
      {
        id: `${sourceKey}:${foreignKey.name}:${foreignKey.column}`,
        sourceKey,
        targetKey,
        label,
        path: `M ${startX} ${startY} C ${startX + (right ? bend : -bend)} ${startY}, ${endX - (right ? bend : -bend)} ${endY}, ${endX} ${endY}`,
      },
    ]
  }),
)
const planeSize = computed(() => {
  const nodes = visibleTables.value
  if (!nodes.length) return { width: 1200, height: 800 }
  return {
    width: Math.max(1200, ...nodes.map((table) => positionOf(table).x + nodeWidth + 100)),
    height: Math.max(800, ...nodes.map((table) => positionOf(table).y + tableHeight(table) + 100)),
  }
})
function schemaColor(schema: string) {
  const schemas = [...new Set(sortedTables.value.map((table) => table.schema))]
  return schemaColors[schemas.indexOf(schema) % schemaColors.length]!
}
function focusTable(table: TableInfo) {
  activeTable.value = tableKey(table)
}
function openTable(table: TableInfo) {
  emit('choose', table)
  dialog.value?.close()
}
function startPan(event: PointerEvent) {
  if (event.button !== 0 || event.target !== event.currentTarget) return
  const element = event.currentTarget as HTMLElement
  element.setPointerCapture(event.pointerId)
  moving.value = {
    kind: 'pan',
    pointerId: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    originX: pan.value.x,
    originY: pan.value.y,
  }
}
function startNodeDrag(event: PointerEvent, table: TableInfo) {
  if (event.button !== 0) return
  const key = tableKey(table)
  const point = positionOf(table)
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
  moving.value = {
    kind: 'node',
    pointerId: event.pointerId,
    key,
    x: event.clientX,
    y: event.clientY,
    originX: point.x,
    originY: point.y,
  }
}
function movePointer(event: PointerEvent) {
  const drag = moving.value
  if (!drag || drag.pointerId !== event.pointerId) return
  if (drag.kind === 'pan') {
    pan.value = {
      x: drag.originX + event.clientX - drag.x,
      y: drag.originY + event.clientY - drag.y,
    }
  } else {
    positions.value[drag.key] = {
      x: Math.max(24, drag.originX + (event.clientX - drag.x) / zoom.value),
      y: Math.max(24, drag.originY + (event.clientY - drag.y) / zoom.value),
    }
  }
}
function stopPointer(event: PointerEvent) {
  if (moving.value?.pointerId === event.pointerId) moving.value = null
}
function changeZoom(amount: number, focusX?: number, focusY?: number) {
  const element = viewport.value
  const pointX = focusX ?? (element?.clientWidth ?? 0) / 2
  const pointY = focusY ?? (element?.clientHeight ?? 0) / 2
  const next = Math.max(0.35, Math.min(1.6, Math.round((zoom.value + amount) * 100) / 100))
  if (next === zoom.value) return
  pan.value = {
    x: pointX - ((pointX - pan.value.x) / zoom.value) * next,
    y: pointY - ((pointY - pan.value.y) / zoom.value) * next,
  }
  zoom.value = next
}
function arrangeFlow() {
  positions.value = { ...flowPositions.value }
  void fitView()
}
function zoomAt(event: WheelEvent) {
  if (!viewport.value) return
  const bounds = viewport.value.getBoundingClientRect()
  changeZoom(
    event.deltaY < 0 ? 0.08 : -0.08,
    event.clientX - bounds.left,
    event.clientY - bounds.top,
  )
}
async function fitView() {
  await nextTick()
  const element = viewport.value
  if (!element) return
  const nodes = visibleTables.value
  if (!nodes.length) {
    zoom.value = 1
    pan.value = { x: 0, y: 0 }
    return
  }
  const minX = Math.min(...nodes.map((table) => positionOf(table).x))
  const minY = Math.min(...nodes.map((table) => positionOf(table).y))
  const maxX = Math.max(...nodes.map((table) => positionOf(table).x + nodeWidth))
  const maxY = Math.max(...nodes.map((table) => positionOf(table).y + tableHeight(table)))
  const boundsWidth = Math.max(1, maxX - minX)
  const boundsHeight = Math.max(1, maxY - minY)
  const scale = Math.max(
    0.35,
    Math.min(
      1.2,
      (element.clientWidth - 100) / boundsWidth,
      (element.clientHeight - 100) / boundsHeight,
    ),
  )
  zoom.value = scale
  pan.value = {
    x: (element.clientWidth - boundsWidth * scale) / 2 - minX * scale,
    y: (element.clientHeight - boundsHeight * scale) / 2 - minY * scale,
  }
}
function closeDialog() {
  dialog.value?.close()
}
function closeFromBackdrop(event: MouseEvent) {
  if (event.target === dialog.value) closeDialog()
}
onMounted(async () => {
  dialog.value?.showModal()
  await fitView()
})
watch(filter, fitView)
watch(
  () => props.tables.length,
  () => fitView(),
)
</script>

<template>
  <dialog
    ref="dialog"
    class="schema-diagram-modal"
    aria-label="Database schema diagram"
    @close="emit('close')"
    @click="closeFromBackdrop"
  >
    <div class="schema-diagram-shell">
      <header class="schema-diagram-header">
        <div class="schema-diagram-title">
          <span class="schema-diagram-mark"><AppIcon name="diagram" :size="21" /></span>
          <div>
            <h2>Database diagram</h2>
            <p>
              <strong>{{ database }}</strong>
              <span>{{ tables.length }} {{ tables.length === 1 ? 'table' : 'tables' }}</span>
              <span
                >{{ relationshipCount }}
                {{ relationshipCount === 1 ? 'relationship' : 'relationships' }}</span
              >
            </p>
          </div>
        </div>
        <div class="schema-diagram-actions">
          <label class="search-box schema-diagram-search">
            <AppIcon name="search" :size="15" />
            <input
              v-model="filter"
              placeholder="Find tables or columns"
              aria-label="Find tables or columns"
            />
            <button
              v-if="filter"
              type="button"
              class="icon-button"
              aria-label="Clear search"
              @click="filter = ''"
            >
              <AppIcon name="close" :size="13" />
            </button>
          </label>
          <div class="diagram-zoom-controls" role="group" aria-label="Diagram zoom controls">
            <button
              type="button"
              class="icon-button"
              title="Zoom out"
              aria-label="Zoom out"
              @click="changeZoom(-0.1)"
            >
              <AppIcon name="zoomOut" :size="18" />
            </button>
            <span class="diagram-zoom-value" aria-live="polite">{{ Math.round(zoom * 100) }}%</span>
            <button
              type="button"
              class="icon-button"
              title="Zoom in"
              aria-label="Zoom in"
              @click="changeZoom(0.1)"
            >
              <AppIcon name="zoomIn" :size="18" />
            </button>
            <button
              type="button"
              class="button secondary small diagram-fit-button"
              title="Arrange tables by their relationships"
              @click="arrangeFlow"
            >
              <AppIcon name="diagram" :size="15" /><span>Arrange</span>
            </button>
            <button
              type="button"
              class="button secondary small diagram-fit-button"
              title="Fit tables to the canvas"
              @click="fitView"
            >
              <AppIcon name="fit" :size="16" /><span>Fit</span>
            </button>
          </div>
          <button
            type="button"
            class="icon-button diagram-close"
            title="Close diagram"
            aria-label="Close diagram"
            @click="closeDialog"
          >
            <AppIcon name="close" :size="18" />
          </button>
        </div>
      </header>

      <div class="schema-diagram-legend" aria-label="Diagram legend">
        <span><AppIcon name="key" :size="13" /> Primary key</span>
        <span><AppIcon name="link" :size="13" /> Foreign key</span>
        <span class="diagram-edge-key"><i /> Declared relationship</span>
        <span
          v-if="relationshipMetadataMissing"
          class="diagram-legend-hint relationship-metadata-warning"
        >
          Relationship metadata is unavailable. Restart WebDB, reconnect, then reopen the diagram.
        </span>
        <span v-else class="diagram-legend-hint"
          >Flow from source tables top to bottom. Drag cards to adjust. Drag empty space to
          pan.</span
        >
      </div>

      <main
        ref="viewport"
        class="schema-diagram-viewport"
        :class="{ panning: moving?.kind === 'pan' }"
        aria-label="Interactive schema diagram canvas"
        @pointerdown="startPan"
        @pointermove="movePointer"
        @pointerup="stopPointer"
        @pointercancel="stopPointer"
        @wheel.prevent="zoomAt"
      >
        <div v-if="loading" class="schema-diagram-empty" role="status">
          <span class="loader" />
          <p>Reading table structure…</p>
        </div>
        <div v-else-if="!tables.length" class="schema-diagram-empty">
          <AppIcon name="table" :size="26" />
          <h3>No accessible tables</h3>
          <p>Refresh the database explorer after granting access to tables.</p>
        </div>
        <div v-else-if="!visibleTables.length" class="schema-diagram-empty">
          <AppIcon name="search" :size="25" />
          <h3>No matching tables</h3>
          <p>Try a different table or column name.</p>
        </div>
        <div
          v-else
          class="schema-diagram-plane"
          :style="{
            width: planeSize.width + 'px',
            height: planeSize.height + 'px',
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
          }"
        >
          <svg
            class="schema-diagram-edges"
            :width="planeSize.width"
            :height="planeSize.height"
            :viewBox="`0 0 ${planeSize.width} ${planeSize.height}`"
            aria-label="Foreign key relationships"
          >
            <defs>
              <marker
                id="diagram-arrow"
                viewBox="0 0 10 10"
                refX="8"
                refY="5"
                markerWidth="6"
                markerHeight="6"
                orient="auto-start-reverse"
              >
                <path d="M 0 0 L 10 5 L 0 10 z" />
              </marker>
            </defs>
            <path
              v-for="edge in edges"
              :key="edge.id"
              :d="edge.path"
              class="schema-diagram-edge"
              :class="{
                emphasized:
                  hoveredTable === edge.sourceKey ||
                  hoveredTable === edge.targetKey ||
                  activeTable === edge.sourceKey ||
                  activeTable === edge.targetKey,
                subdued:
                  !!(hoveredTable || activeTable) &&
                  hoveredTable !== edge.sourceKey &&
                  hoveredTable !== edge.targetKey &&
                  activeTable !== edge.sourceKey &&
                  activeTable !== edge.targetKey,
              }"
            >
              <title>{{ edge.label }}</title>
            </path>
          </svg>

          <article
            v-for="table in visibleTables"
            :key="tableKey(table)"
            class="schema-diagram-table"
            :class="{
              'is-current':
                selectedTable?.schema === table.schema && selectedTable.name === table.name,
              'is-active': activeTable === tableKey(table),
              'is-related': isRelated(table),
            }"
            :style="{
              left: positionOf(table).x + 'px',
              top: positionOf(table).y + 'px',
              '--schema-accent': schemaColor(table.schema),
            }"
            @mouseenter="hoveredTable = tableKey(table)"
            @mouseleave="hoveredTable = null"
          >
            <button
              type="button"
              class="schema-diagram-table-heading"
              :aria-label="`Select ${table.schema}.${table.name}`"
              @pointerdown.stop="startNodeDrag($event, table)"
              @click="focusTable(table)"
            >
              <span class="schema-table-grip" aria-hidden="true"
                ><i /><i /><i /><i /><i /><i
              /></span>
              <span class="schema-table-heading-copy">
                <small>{{ table.schema }}</small>
                <strong>{{ table.name }}</strong>
              </span>
              <span class="schema-table-kind">{{
                table.kind.toLowerCase().includes('view') ? 'VIEW' : 'TABLE'
              }}</span>
            </button>
            <div class="schema-diagram-columns">
              <div
                v-for="column in table.columns.slice(0, shownColumns)"
                :key="column"
                class="schema-diagram-column"
              >
                <span class="schema-column-mark">
                  <AppIcon v-if="primaryKeys(table).includes(column)" name="key" :size="13" />
                  <AppIcon v-else-if="isForeignKey(table, column)" name="link" :size="13" />
                  <i v-else />
                </span>
                <span
                  class="schema-column-name"
                  :class="{ 'is-primary': primaryKeys(table).includes(column) }"
                  >{{ column }}</span
                >
                <span v-if="primaryKeys(table).includes(column)" class="schema-column-badge primary"
                  >PK</span
                >
                <span v-else-if="isForeignKey(table, column)" class="schema-column-badge foreign"
                  >FK</span
                >
              </div>
              <div v-if="table.columns.length > shownColumns" class="schema-column-more">
                and {{ table.columns.length - shownColumns }} more
                {{ table.columns.length - shownColumns === 1 ? 'column' : 'columns' }}
              </div>
              <p v-if="!table.columns.length" class="schema-columns-empty">No column metadata</p>
            </div>
            <footer class="schema-diagram-table-footer">
              <span
                >{{ table.columns.length }}
                {{ table.columns.length === 1 ? 'column' : 'columns' }}</span
              >
              <button
                type="button"
                :aria-label="`Open ${table.schema}.${table.name}`"
                @click="openTable(table)"
              >
                Open table <AppIcon name="arrow" :size="13" />
              </button>
            </footer>
          </article>
        </div>
      </main>

      <footer class="schema-diagram-footer">
        <span>Scroll to zoom · Use Fit to bring all visible tables into view</span>
        <span
          >{{ visibleTables.length }} of {{ tables.length }} tables <i /> {{ edges.length }} visible
          relationships</span
        >
      </footer>
    </div>
  </dialog>
</template>

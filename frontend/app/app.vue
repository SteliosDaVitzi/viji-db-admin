<script setup lang="ts">
import { providerOptions, quoteIdentifier } from './providers'
import type { ColumnInfo, ConnectionInput, ContextAction, TableInfo } from '../shared/database'
const db = useDatabase()
const editor = ref<'insert' | 'delete' | 'schema' | null>(null)
const editingCell = ref<{
  row: Record<string, unknown>
  column: ColumnInfo
} | null>(null)
const editable = computed(
  () =>
    !!session.value &&
    session.value.provider !== 'demo' &&
    providerOptions(session.value.provider).editing &&
    !activeQuery.value &&
    selectedTable.value?.kind === 'BASE TABLE',
)
const recordEditable = computed(
  () =>
    editable.value &&
    (session.value?.provider === 'postgres' ||
      !!data.value?.columns.some((column) => column.primaryKey)),
)
const mutationNotice = ref('')
const settingsOpen = ref(false)
const context = shallowRef<{
  x: number
  y: number
  label: string
  actions: ContextAction[]
} | null>(null)
function showContext(event: MouseEvent, label: string, actions: ContextAction[]) {
  context.value = { x: event.clientX, y: event.clientY, label, actions }
}
async function copy(value: unknown) {
  try {
    await navigator.clipboard.writeText(
      typeof value === 'string' ? value : JSON.stringify(value, null, 2),
    )
    mutationNotice.value = 'Copied to clipboard.'
  } catch {
    mutationNotice.value = 'Clipboard access failed. Check your browser permissions.'
  }
}
function workspaceContext(event: MouseEvent) {
  showContext(event, 'Workspace', [
    {
      label: 'Refresh explorer',
      disabled: !session.value || busy.value || switching.value,
      run: db.refreshExplorer,
    },
    {
      label: 'Database settings',
      disabled: session.value?.provider !== 'postgres' || switching.value,
      run: () => {
        settingsOpen.value = true
      },
    },
  ])
}
function tableContext(event: MouseEvent, table: TableInfo) {
  const disabled = busy.value || switching.value
  showContext(event, table.name, [
    { label: 'Browse table', disabled, run: () => db.chooseTable(table) },
    {
      label: 'View structure',
      disabled,
      run: async () => {
        await db.chooseTable(table)
        tab.value = 'structure'
      },
    },
    {
      label: 'Copy table name',
      run: () =>
        copy(
          `${quoteIdentifier(table.schema, session.value?.provider)}.${quoteIdentifier(table.name, session.value?.provider)}`,
        ),
    },
    { label: 'Refresh explorer', disabled, run: db.refreshExplorer },
  ])
}
function schemaContext(event: MouseEvent, schema: string) {
  showContext(event, schema, [
    { label: 'Copy schema name', run: () => copy(schema) },
    { label: 'Refresh explorer', disabled: switching.value || busy.value, run: db.refreshExplorer },
  ])
}
function rowContext(event: MouseEvent, index: number, column?: string) {
  const row = data.value?.rows[index]
  if (!row) return
  selectedRow.value = index
  const actions: ContextAction[] = [
    { label: 'Copy row as JSON', run: () => copy(row) },
    { label: 'Export current page', run: exportPage },
  ]
  if (column)
    actions.unshift(
      { label: 'Inspect cell', run: () => editCell(index, column) },
      { label: 'Copy cell value', run: () => copy(row[column]) },
    )
  if (editable.value)
    actions.push(
      {
        label: 'Insert record',
        run: () => {
          editor.value = 'insert'
        },
      },
      {
        disabled: !recordEditable.value,
        label: 'Delete row…',
        run: () => {
          editor.value = 'delete'
        },
      },
    )
  const number = page.value * pageSize.value + index + 1
  showContext(event, column ? `${column} · Row ${number}` : `Row ${number}`, actions)
}
async function mutationSaved() {
  editor.value = null
  selectedRow.value = null
  queryViews.clear()
  mutationNotice.value = 'Change applied successfully.'
  explorerKey.value++
  await db.refreshExplorer()
}
const {
  providers,
  session,
  selectedTable,
  data,
  selectedRow,
  busy,
  connecting,
  connectionError,
  error,
  page,
  pageSize,
  sort,
  descending,
  activeQuery,
  databases,
  tables,
  tablesLoading,
  savedConnections,
  savingEnabled,
  switching,
} = db
const sidebar = useSidebarResize()
const explorerKey = ref(0)
const rowFilter = ref('')
const tab = ref<'data' | 'structure'>('data')
const queryWorkspaceKey = ref(0)
let currentQueryId = 'query-1'
function captureQueryView() {
  return {
    data: data.value,
    activeQuery: activeQuery.value,
    page: page.value,
    pageSize: pageSize.value,
    sort: sort.value,
    descending: descending.value,
    table: selectedTable.value,
    filter: rowFilter.value,
    tab: tab.value,
    error: error.value,
  }
}
const queryViews = new Map<string, ReturnType<typeof captureQueryView>>()
watch(session, () => {
  settingsOpen.value = false
  context.value = null
  editingCell.value = null
  editor.value = null
  mutationNotice.value = ''
  queryViews.clear()
  currentQueryId = 'query-1'
  queryWorkspaceKey.value++
})
async function activateQuery(id: string) {
  if (busy.value || id === currentQueryId) return
  queryViews.set(currentQueryId, captureQueryView())
  currentQueryId = id
  selectedRow.value = null
  const view = queryViews.get(id)
  if (view) {
    selectedTable.value = view.table
    data.value = view.data
    activeQuery.value = view.activeQuery
    page.value = view.page
    pageSize.value = view.pageSize
    sort.value = view.sort
    descending.value = view.descending
    error.value = view.error
    await nextTick()
    rowFilter.value = view.filter
    tab.value = view.tab
  } else {
    rowFilter.value = ''
    tab.value = 'data'
    await db.browseTable()
  }
}
const connectionDialog = useTemplateRef<HTMLDialogElement>('connectionDialog')
const initialSql = computed(() => {
  return selectedTable.value
    ? `SELECT * FROM ${quoteIdentifier(selectedTable.value.schema, session.value?.provider)}.${quoteIdentifier(selectedTable.value.name, session.value?.provider)};`
    : 'SELECT 1 AS result;'
})
async function runQuery(sql: string) {
  tab.value = 'data'
  rowFilter.value = ''
  await db.runQuery(sql)
}
const record = computed(() =>
  selectedRow.value === null ? null : data.value?.rows[selectedRow.value] || null,
)
watch([data, busy, switching], () => {
  context.value = null
})
watch(selectedTable, () => {
  rowFilter.value = ''
  tab.value = 'data'
})
async function connect(input: ConnectionInput | string) {
  if (await db.connect(input)) {
    explorerKey.value++
    connectionDialog.value?.close()
  }
}
function exportPage() {
  if (!data.value) return
  const blob = new Blob([JSON.stringify(data.value.rows, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${activeQuery.value ? 'query-results' : selectedTable.value?.name || 'table'}-page-${page.value + 1}.json`
  link.click()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
watch(rowFilter, () => {
  selectedRow.value = null
})
function editCell(index: number, name: string) {
  selectedRow.value = index
  const row = data.value?.rows[index]
  const column = data.value?.columns.find((column) => column.name === name)
  if (row && column) editingCell.value = { row, column }
}
async function cellSaved() {
  editingCell.value = null
  queryViews.clear()
  mutationNotice.value = 'Value saved.'
  await db.loadRows()
}
</script>
<template>
  <div class="workspace" @contextmenu.prevent="workspaceContext">
    <header class="app-header">
      <a href="/" class="brand" aria-label="WebDB home"
        ><span class="brand-icon"><AppIcon name="database" :size="20" /></span>web<span
          class="brand-db"
          >db</span
        ><span class="brand-caption">Database workspace</span></a
      >
      <div v-if="session" class="header-actions">
        <span class="read-only"
          ><AppIcon name="lock" :size="13" />{{
            session && session.provider !== 'demo' ? 'Browse & edit' : 'Read only'
          }}</span
        ><button
          v-if="session"
          class="button secondary small"
          @click="connectionDialog?.showModal()"
        >
          <AppIcon name="plus" :size="15" /><span>New connection</span>
        </button>
      </div>
    </header>
    <div
      class="app-body"
      :class="{ disconnected: !session, 'resizing-sidebar': sidebar.resizing.value }"
      :style="{ '--sidebar-width': sidebar.width.value + 'px' }"
    >
      <aside v-if="session" id="database-sidebar" class="sidebar" aria-label="Database explorer">
        <div class="sidebar-label">Connection</div>
        <div class="connection-card">
          <div class="connection-summary">
            <span class="provider-mark"><AppIcon name="database" :size="19" /></span>
            <div>
              <strong>{{
                session?.allDatabases ? session.host : session?.database || 'No connection'
              }}</strong
              ><span>{{
                session
                  ? session.provider === 'demo'
                    ? 'Sample workspace'
                    : session.allDatabases
                      ? `${providers.find((p) => p.id === session?.provider)?.name ?? session.provider} instance`
                      : `${providers.find((p) => p.id === session?.provider)?.name ?? session.provider} / ${session.host}`
                  : 'Connect to get started'
              }}</span>
            </div>
            <span
              class="status-dot"
              :class="{ online: session }"
              :title="session ? 'Connected' : 'Disconnected'"
            />
          </div>
          <div v-if="session" class="database-select-row">
            <DatabaseSelect
              v-if="session"
              :databases="session.allDatabases ? databases : [session.database]"
              :model-value="session.database"
              :disabled="connecting || switching || tablesLoading"
              @update:model-value="db.switchDatabase($event)"
            />
            <button
              v-if="session.provider === 'postgres'"
              class="icon-button database-settings-button"
              title="Database settings"
              aria-label="Database settings"
              :disabled="connecting || switching"
              @click="settingsOpen = true"
            >
              <AppIcon name="settings" :size="16" />
            </button>
          </div>
        </div>
        <DatabaseExplorer
          v-if="session"
          :key="explorerKey"
          :tables="tables"
          :loading="tablesLoading"
          :active-database="session.database"
          :selected-table="selectedTable"
          :disabled="connecting || switching"
          @choose="db.chooseDatabaseTable"
          @refresh="db.refreshExplorer"
          @context="tableContext"
          @schema-context="schemaContext"
        />
        <div v-else class="sidebar-placeholder">
          <AppIcon name="table" :size="22" />
          <p>Your tables will<br />appear here.</p>
        </div>
        <div class="sidebar-footer">
          <template v-if="session"
            ><span
              ><span class="status-dot online" />{{
                session.provider === 'demo' ? 'Sample data' : 'Connected'
              }}</span
            ><button
              class="icon-button"
              title="Disconnect"
              aria-label="Disconnect"
              :disabled="connecting || switching"
              @click="db.disconnect"
            >
              <AppIcon name="disconnect" :size="15" /></button></template
          ><template v-else
            ><AppIcon name="lock" :size="13" /><span>Your database workspace</span></template
          >
        </div>
      </aside>
      <div
        v-if="session"
        class="sidebar-resizer"
        role="separator"
        tabindex="0"
        aria-label="Resize database explorer"
        aria-controls="database-sidebar"
        aria-orientation="vertical"
        :aria-valuemin="sidebar.minimum.value"
        :aria-valuemax="sidebar.maximum.value"
        :aria-valuenow="sidebar.width.value"
        :aria-valuetext="sidebar.width.value + ' pixels'"
        title="Drag to resize. Arrow keys resize; double-click resets."
        @pointerdown="sidebar.start"
        @pointermove="sidebar.move"
        @pointerup="sidebar.stop"
        @pointercancel="sidebar.stop"
        @lostpointercapture="sidebar.stop"
        @keydown="sidebar.keydown"
        @dblclick="sidebar.reset"
      />
      <main v-if="!session" class="welcome-main">
        <ConnectionForm
          :providers="providers"
          :busy="connecting"
          :error="connectionError"
          :saved-connections="savedConnections"
          :saving-enabled="savingEnabled"
          @reconnect="connect"
          @remove="db.deleteSavedConnection"
          @connect="connect"
        />
      </main>
      <main v-else class="database-main">
        <QueryEditor
          :key="queryWorkspaceKey"
          :initial-sql="initialSql"
          :provider="session.provider"
          :busy="busy || switching"
          :active-query="activeQuery"
          :demo="session.provider === 'demo'"
          :tables="tables"
          :selected-table="selectedTable"
          @run="runQuery"
          @activate="activateQuery"
          @close="queryViews.delete($event)"
        />
        <div class="table-heading">
          <AppIcon :name="activeQuery ? 'grid' : 'table'" :size="15" />
          <h1>{{ activeQuery ? 'Query results' : selectedTable?.name || 'Your database' }}</h1>
          <span class="table-subtitle"
            >{{ session.database }} / {{ selectedTable?.schema || 'Tables' }}</span
          >
          <div v-if="session.provider !== 'demo'" class="mutation-toolbar">
            <button
              v-if="editable"
              class="button secondary small"
              :disabled="busy || switching"
              @click="editor = 'insert'"
            >
              Insert record
            </button>
            <button
              class="button secondary small"
              :disabled="busy || switching"
              @click="editor = 'schema'"
            >
              Database changes
            </button>
          </div>
        </div>
        <p v-if="mutationNotice" class="mutation-notice" role="status">
          {{ mutationNotice }}
          <button class="text-button" @click="mutationNotice = ''">Dismiss</button>
        </p>
        <div class="view-toolbar">
          <div class="tabs" role="tablist" aria-label="Table view">
            <button
              id="data-tab"
              role="tab"
              :aria-selected="tab === 'data'"
              :class="{ active: tab === 'data' }"
              @click="tab = 'data'"
              @keydown.right="tab = 'structure'"
            >
              <AppIcon name="table" :size="15" />{{ activeQuery ? 'Results' : 'Data'
              }}<span v-if="data">{{ data.rows.length }}</span></button
            ><button
              id="structure-tab"
              role="tab"
              :aria-selected="tab === 'structure'"
              :class="{ active: tab === 'structure' }"
              @click="tab = 'structure'"
              @keydown.left="tab = 'data'"
            >
              <AppIcon name="grid" :size="15" />Structure<span v-if="data">{{
                data.columns.length
              }}</span>
            </button>
          </div>
          <label v-if="tab === 'data'" class="search-box row-search"
            ><AppIcon name="search" :size="14" /><input
              v-model="rowFilter"
              placeholder="Filter this page…"
              aria-label="Filter rows on this page" /><button
              v-if="rowFilter"
              class="icon-button"
              title="Clear filter"
              aria-label="Clear filter"
              @click="rowFilter = ''"
            >
              <AppIcon name="close" :size="12" /></button
          ></label>
          <div class="table-heading-actions">
            <button
              v-if="recordEditable && record && tab === 'data'"
              class="button secondary small delete-row-button"
              :disabled="busy || switching"
              @click="editor = 'delete'"
            >
              Delete row
            </button>
            <button
              class="icon-button"
              title="Refresh rows"
              aria-label="Refresh rows"
              :disabled="busy || (!selectedTable && !activeQuery)"
              @click="db.loadRows"
            >
              <AppIcon name="refresh" :size="17" :class="{ spinning: busy }" /></button
            ><button
              class="icon-button"
              title="Export current page as JSON"
              aria-label="Export current page as JSON"
              :disabled="!data"
              @click="exportPage"
            >
              <AppIcon name="download" :size="17" />
            </button>
          </div>
          <button
            v-if="activeQuery"
            class="text-button back-to-table"
            :disabled="busy"
            @click="db.browseTable"
          >
            Back to table
          </button>
          <div v-if="data" class="pagination toolbar-pagination">
            <span
              >{{
                data.rows.length
                  ? `${page * pageSize + 1}–${page * pageSize + data.rows.length}`
                  : '0'
              }}
              {{ activeQuery ? 'result rows' : 'rows'
              }}<span class="pagination-detail"> · {{ data.columns.length }} columns</span></span
            >
            <div>
              <label
                >Rows per page<select
                  :value="pageSize"
                  aria-label="Rows per page"
                  :disabled="busy"
                  @change="db.changePageSize(Number(($event.target as HTMLSelectElement).value))"
                >
                  <option :value="25">25</option>
                  <option :value="50">50</option>
                  <option :value="100">100</option>
                </select></label
              ><button
                class="icon-button previous"
                title="Previous page"
                aria-label="Previous page"
                :disabled="page === 0 || busy"
                @click="db.changePage(page - 1)"
              >
                <AppIcon name="chevron" :size="15" /></button
              ><span class="page-number">{{ page + 1 }}</span
              ><button
                class="icon-button"
                title="Next page"
                aria-label="Next page"
                :disabled="!data.hasMore || busy"
                @click="db.changePage(page + 1)"
              >
                <AppIcon name="chevron" :size="15" />
              </button>
            </div>
          </div>
        </div>
        <div v-if="error" class="error-banner" role="alert">
          {{ error
          }}<button
            class="text-button"
            @click="selectedTable || activeQuery ? db.loadRows() : db.loadTables()"
          >
            Try again
          </button>
        </div>
        <div id="table-grid-region" class="data-body">
          <section
            class="data-content"
            role="tabpanel"
            :aria-labelledby="tab === 'data' ? 'data-tab' : 'structure-tab'"
            :aria-busy="busy"
          >
            <div v-if="busy" class="content-empty" role="status">
              <span class="loader" />
              <p>Reading your table…</p>
            </div>
            <template v-else-if="data"
              ><DataGrid
                v-if="tab === 'data'"
                :data="data"
                :sort="sort"
                :descending="descending"
                :filter="rowFilter"
                :sortable="!activeQuery"
                :selected-row="selectedRow"
                @select="selectedRow = $event"
                @edit="editCell"
                @sort="db.sortBy"
                @context="rowContext"
              />
              <div v-else class="grid-scroll">
                <table class="structure-grid">
                  <thead>
                    <tr>
                      <th>Column</th>
                      <th>Type</th>
                      <th>Nullable</th>
                      <th>Key</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="column in data.columns" :key="column.name">
                      <td>{{ column.name }}</td>
                      <td>{{ column.dataType }}</td>
                      <td>{{ column.nullable ? 'Yes' : 'No' }}</td>
                      <td>
                        <span v-if="column.primaryKey" class="key-label"
                          ><AppIcon name="key" :size="13" />Primary key</span
                        ><span v-else class="muted">—</span>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div></template
            >
            <div v-else class="content-empty">
              <AppIcon name="table" :size="32" />
              <h2>{{ error ? 'Unable to load this table' : 'Ready when you are' }}</h2>
              <p>
                {{
                  error
                    ? 'Review the message above and try again.'
                    : 'Select a table from the explorer.'
                }}
              </p>
            </div>
          </section>
        </div>
      </main>
    </div>
    <footer class="app-footer">
      <a class="footer-brand" href="https://vijitek.com">
        <span>By</span>
        <span class="footer-logo" aria-hidden="true">
          <img
            src="~/assets/images/vijitek-logo.webp"
            alt=""
            width="1280"
            height="1280"
            decoding="async"
          />
        </span>
        <strong>Vijitek</strong>
      </a>
    </footer>
    <ContextMenu
      v-if="context"
      :key="`${context.x}-${context.y}-${context.label}`"
      v-bind="context"
      @close="context = null"
    />
    <DatabaseSettings
      v-if="settingsOpen && session"
      :database="session.database"
      :provider="session.provider"
      @close="settingsOpen = false"
    />
    <CellEditor
      v-if="editingCell"
      :column="editingCell.column"
      :row="editingCell.row"
      :table="selectedTable"
      :editable="recordEditable"
      :readonly-reason="
        editable && !recordEditable
          ? 'Updating records requires a primary key for this provider.'
          : undefined
      "
      :apply="db.applyMutation"
      @close="editingCell = null"
      @saved="cellSaved"
    />
    <MutationDialog
      v-if="editor && session"
      :mode="editor"
      :provider="session.provider"
      :table="selectedTable"
      :columns="data?.columns || []"
      :row="record"
      :database="session.database"
      :instance="session.allDatabases"
      :apply="db.applyMutation"
      @close="editor = null"
      @saved="mutationSaved"
    />
    <dialog ref="connectionDialog" class="connection-dialog" aria-label="Connect to a database">
      <button
        class="icon-button dialog-close"
        title="Close connection dialog"
        aria-label="Close connection dialog"
        @click="connectionDialog?.close()"
      >
        <AppIcon name="close" /></button
      ><ConnectionForm
        :providers="providers"
        :busy="connecting"
        :error="connectionError"
        :saved-connections="savedConnections"
        :saving-enabled="savingEnabled"
        @reconnect="connect"
        @remove="db.deleteSavedConnection"
        @connect="connect"
      />
    </dialog>
  </div>
</template>

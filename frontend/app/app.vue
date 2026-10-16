<script setup lang="ts">
import type { ConnectionInput } from '../shared/database'
const db = useDatabase()
const {
  providers,
  session,
  tables,
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
} = db
const tableFilter = ref('')
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
  const quote = (name: string) => `"${name.replaceAll('"', '""')}"`
  return selectedTable.value
    ? `SELECT * FROM ${quote(selectedTable.value.schema)}.${quote(selectedTable.value.name)};`
    : 'SELECT 1 AS result;'
})
async function runQuery(sql: string) {
  tab.value = 'data'
  rowFilter.value = ''
  await db.runQuery(sql)
}
const filteredTables = computed(() =>
  tables.value.filter((t) =>
    `${t.schema}.${t.name}`.toLowerCase().includes(tableFilter.value.toLowerCase()),
  ),
)
const schemas = computed(() => [...new Set(filteredTables.value.map((t) => t.schema))])
const record = computed(() =>
  selectedRow.value === null ? null : data.value?.rows[selectedRow.value] || null,
)
watch(selectedTable, () => {
  rowFilter.value = ''
  tab.value = 'data'
})
async function connect(input: ConnectionInput) {
  if (await db.connect(input)) connectionDialog.value?.close()
}
async function demo() {
  await connect({
    provider: 'demo',
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl: true,
  })
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
function inspectRow(index: number) {
  selectedRow.value = index
}
</script>
<template>
  <div class="workspace">
    <header class="app-header">
      <a href="/" class="brand" aria-label="WebDB home"
        ><span class="brand-icon"><AppIcon name="database" :size="20" /></span>web<span
          class="brand-db"
          >db</span
        ><span class="brand-caption">Database viewer</span></a
      >
      <div class="header-actions">
        <span class="read-only"><AppIcon name="lock" :size="13" />Read only</span
        ><button
          v-if="session"
          class="button secondary small"
          @click="connectionDialog?.showModal()"
        >
          <AppIcon name="plus" :size="15" /><span>New connection</span></button
        ><span v-else class="version">Workspace / 01</span>
      </div>
    </header>
    <div class="app-body" :class="{ disconnected: !session }">
      <aside class="sidebar" aria-label="Database tables">
        <div class="sidebar-label">Connection</div>
        <div class="connection-summary">
          <span class="provider-mark"><AppIcon name="database" :size="19" /></span>
          <div>
            <strong>{{ session?.database || 'No connection' }}</strong
            ><span>{{
              session
                ? session.provider === 'demo'
                  ? 'Sample workspace'
                  : `PostgreSQL · ${session.host}`
                : 'Connect to get started'
            }}</span>
          </div>
          <span
            class="status-dot"
            :class="{ online: session }"
            :title="session ? 'Connected' : 'Disconnected'"
          />
        </div>
        <template v-if="session"
          ><div class="sidebar-section-heading">
            <span>Explorer</span
            ><button
              class="icon-button"
              title="Refresh table list"
              aria-label="Refresh table list"
              :disabled="busy || connecting"
              @click="db.loadTables"
            >
              <AppIcon name="refresh" :size="14" />
            </button>
          </div>
          <label class="search-box table-search"
            ><AppIcon name="search" :size="15" /><input
              v-model="tableFilter"
              placeholder="Find a table…"
              aria-label="Find a table"
            /><kbd>/</kbd></label
          >
          <nav class="table-list" aria-label="Tables">
            <div v-for="schema in schemas" :key="schema" class="schema-group">
              <div class="schema-label">
                <AppIcon name="down" :size="12" />{{ schema
                }}<span>{{ filteredTables.filter((t) => t.schema === schema).length }}</span>
              </div>
              <button
                v-for="table in filteredTables.filter((t) => t.schema === schema)"
                :key="`${table.schema}.${table.name}`"
                class="table-link"
                :class="{
                  active:
                    selectedTable?.schema === table.schema && selectedTable?.name === table.name,
                }"
                :aria-current="
                  selectedTable?.schema === table.schema && selectedTable?.name === table.name
                    ? 'true'
                    : undefined
                "
                @click="db.chooseTable(table)"
              >
                <AppIcon name="table" :size="15" /><span>{{ table.name }}</span
                ><AppIcon
                  v-if="
                    selectedTable?.name === table.name && selectedTable?.schema === table.schema
                  "
                  name="chevron"
                  :size="12"
                />
              </button>
            </div>
            <p v-if="!filteredTables.length" class="sidebar-empty">
              {{ tableFilter ? 'No matching tables.' : 'No accessible tables found.' }}
            </p>
          </nav></template
        >
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
              @click="db.disconnect"
            >
              <AppIcon name="disconnect" :size="15" /></button></template
          ><template v-else
            ><AppIcon name="lock" :size="13" /><span>A read-only workspace</span></template
          >
        </div>
      </aside>
      <main v-if="!session" class="welcome-main">
        <div class="welcome-context">
          <span class="context-line" />A clear view of what’s inside.
        </div>
        <ConnectionForm
          :providers="providers"
          :busy="connecting"
          :error="connectionError"
          @connect="connect"
          @demo="demo"
        />
        <div class="welcome-footer">
          <span>PostgreSQL, to start.</span><span>Built for a closer look.</span>
        </div>
      </main>
      <main v-else class="database-main">
        <QueryEditor
          :key="queryWorkspaceKey"
          :initial-sql="initialSql"
          :busy="busy"
          :active-query="activeQuery"
          :demo="session.provider === 'demo'"
          @run="runQuery"
          @activate="activateQuery"
          @close="queryViews.delete($event)"
        />
        <div class="table-heading">
          <AppIcon :name="activeQuery ? 'grid' : 'table'" :size="15" />
          <h1>{{ activeQuery ? 'Query results' : selectedTable?.name || 'Your database' }}</h1>
          <span class="table-subtitle"
            >{{ session.database }} / {{ selectedTable?.schema || 'public' }}</span
          >
        </div>
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
        <div class="data-body">
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
                @inspect="inspectRow"
                @sort="db.sortBy"
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
          <RowInspector
            v-if="record && tab === 'data'"
            :row="record"
            :columns="data?.columns || []"
            :row-number="selectedRow === null ? null : page * pageSize + selectedRow + 1"
            @close="selectedRow = null"
          />
        </div>
      </main>
    </div>
    <footer class="statusbar">
      <span
        ><span class="status-dot" :class="{ online: session }" />{{
          session ? 'Workspace ready' : 'Not connected'
        }}</span
      ><span v-if="session?.provider === 'demo'">Sample data · No database connection</span
      ><span v-else
        >Read-only access<span class="statusbar-divider">/</span>Session-only credentials</span
      ><span>webdb <span class="muted">v0.1</span></span>
    </footer>
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
        @connect="connect"
        @demo="demo"
      />
    </dialog>
  </div>
</template>

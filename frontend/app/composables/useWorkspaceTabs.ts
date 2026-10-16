import { computed, ref, watch } from 'vue'
import type { TableInfo } from '../../shared/database'
import type { useDatabase } from './useDatabase'

// Views live only in this connection's memory; row data is never persisted.
export function useWorkspaceTabs(db: ReturnType<typeof useDatabase>) {
  const rowFilter = ref('')
  const tab = ref<'data' | 'structure'>('data')
  const tableTabs = ref<TableInfo[]>([])
  const activeView = ref('')
  let currentQueryId = 'query-1'
  let restoring = false
  const key = (table: TableInfo) => JSON.stringify([table.schema, table.name])
  const blocked = () => db.busy.value || db.switching.value || db.connecting.value
  function capture() {
    return {
      table: db.selectedTable.value,
      data: db.data.value,
      query: db.activeQuery.value,
      page: db.page.value,
      pageSize: db.pageSize.value,
      sort: db.sort.value,
      descending: db.descending.value,
      error: db.error.value,
      filter: rowFilter.value,
      tab: tab.value,
    }
  }
  const views = new Map<string, ReturnType<typeof capture>>()
  function save() {
    if (activeView.value) views.set(activeView.value, capture())
  }
  function reset() {
    views.clear()
    tableTabs.value = []
    activeView.value = ''
    currentQueryId = 'query-1'
    rowFilter.value = ''
    tab.value = 'data'
  }
  watch(db.session, reset, { flush: 'sync' })
  watch(
    db.selectedTable,
    (table) => {
      if (restoring) return
      rowFilter.value = ''
      tab.value = 'data'
      if (table && !db.activeQuery.value) {
        if (!tableTabs.value.some((item) => key(item) === key(table))) tableTabs.value.push(table)
        activeView.value = key(table)
      }
    },
    { flush: 'sync' },
  )
  watch(db.tables, (tables) => {
    const available = new Set(tables.map(key))
    for (const table of tableTabs.value) {
      if (!available.has(key(table))) views.delete(key(table))
    }
    tableTabs.value = tableTabs.value.filter((table) => available.has(key(table)))
  })
  async function restore(id: string) {
    const view = views.get(id)
    if (!view) return false
    restoring = true
    db.selectedTable.value = view.table
    restoring = false
    db.data.value = view.data
    db.activeQuery.value = view.query
    db.page.value = view.page
    db.pageSize.value = view.pageSize
    db.sort.value = view.sort
    db.descending.value = view.descending
    db.error.value = view.error
    db.selectedRow.value = null
    rowFilter.value = view.filter
    tab.value = view.tab
    activeView.value = id
    if (!view.data && !view.error && (view.query || !id.startsWith('query-'))) await db.loadRows()
    return true
  }
  async function openTable(table: TableInfo) {
    if (blocked()) return
    const id = key(table)
    if (id === activeView.value) return
    save()
    if (await restore(id)) return
    activeView.value = id
    rowFilter.value = ''
    tab.value = 'data'
    if (!tableTabs.value.some((item) => key(item) === id)) tableTabs.value.push(table)
    await db.chooseTable(table)
  }
  async function chooseDatabaseTable(database: string, table: TableInfo) {
    if (blocked()) return
    if (database !== db.session.value?.database && db.session.value?.allDatabases)
      await db.switchDatabase(database, table)
    else await openTable(table)
  }
  async function activateQuery(id: string) {
    if (blocked() || activeView.value === id) return
    save()
    currentQueryId = id
    if (await restore(id)) return
    activeView.value = id
    db.data.value = null
    db.activeQuery.value = null
    db.selectedRow.value = null
    db.error.value = ''
    db.page.value = 0
    rowFilter.value = ''
    tab.value = 'data'
  }
  async function runQuery(sql: string) {
    if (blocked()) return
    if (activeView.value !== currentQueryId) {
      save()
      activeView.value = currentQueryId
    }
    tab.value = 'data'
    rowFilter.value = ''
    await db.runQuery(sql)
  }
  async function backToTable() {
    if (db.selectedTable.value) await openTable(db.selectedTable.value)
  }
  async function closeTable(table: TableInfo) {
    if (blocked()) return
    const id = key(table)
    const index = tableTabs.value.findIndex((item) => key(item) === id)
    const wasActive = activeView.value === id
    tableTabs.value = tableTabs.value.filter((item) => key(item) !== id)
    if (wasActive) {
      const neighbor = tableTabs.value[Math.min(index, tableTabs.value.length - 1)]
      if (neighbor) await openTable(neighbor)
      else {
        activeView.value = ''
        db.selectedTable.value = null
        db.data.value = null
        db.activeQuery.value = null
        db.selectedRow.value = null
        db.error.value = ''
        rowFilter.value = ''
        tab.value = 'data'
      }
    }
    views.delete(id)
  }
  function invalidate() {
    for (const view of views.values()) view.data = null
  }
  return {
    rowFilter,
    tab,
    tableTabs,
    activeView,
    key,
    openTable,
    closeTable,
    chooseDatabaseTable,
    activateQuery,
    runQuery,
    backToTable,
    invalidate,
    closeQuery: (id: string) => views.delete(id),
    queryActive: computed(() => activeView.value.startsWith('query-')),
  }
}

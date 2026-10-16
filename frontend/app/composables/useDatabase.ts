import type {
  ConnectionInput,
  Provider,
  Session,
  TableInfo,
  TablePage,
} from '../../shared/database'

export function useDatabase() {
  const providers = ref<Provider[]>([])
  const session = ref<Session | null>(null)
  const tables = ref<TableInfo[]>([])
  const selectedTable = ref<TableInfo | null>(null)
  const data = shallowRef<TablePage | null>(null)
  const selectedRow = ref<number | null>(null)
  const busy = ref(false)
  const connecting = ref(false)
  const connectionError = ref('')
  const error = ref('')
  const page = ref(0)
  const pageSize = ref(50)
  const sort = ref<string | undefined>()
  const descending = ref(false)
  const activeQuery = ref<string | null>(null)
  let requestVersion = 0
  let controller: AbortController | undefined

  function message(cause: unknown) {
    const failure = cause as { data?: { message?: string }; statusCode?: number }
    if (failure.statusCode === 401) session.value = null
    return (
      failure.data?.message ||
      'Could not reach the database service. Check that the backend is running and try again.'
    )
  }
  async function loadRows() {
    const table = selectedTable.value
    if (!table && !activeQuery.value) return
    const version = ++requestVersion
    controller?.abort()
    controller = new AbortController()
    busy.value = true
    error.value = ''
    selectedRow.value = null
    data.value = null
    try {
      const result = activeQuery.value
        ? await $fetch<TablePage>('/api/query', {
            method: 'POST',
            body: { sql: activeQuery.value, page: page.value, pageSize: pageSize.value },
            signal: controller.signal,
          })
        : await $fetch<TablePage>('/api/rows', {
            query: {
              schema: table!.schema,
              table: table!.name,
              page: page.value,
              pageSize: pageSize.value,
              sort: sort.value,
              descending: descending.value,
            },
            signal: controller.signal,
          })
      if (version === requestVersion) data.value = result
    } catch (cause) {
      if (version === requestVersion) error.value = message(cause)
    } finally {
      if (version === requestVersion) busy.value = false
    }
  }
  async function chooseTable(table: TableInfo) {
    activeQuery.value = null
    selectedTable.value = table
    page.value = 0
    sort.value = undefined
    descending.value = false
    await loadRows()
  }
  async function loadTables() {
    try {
      const currentSession = session.value
      const result = await $fetch<TableInfo[]>('/api/tables')
      if (session.value !== currentSession) return
      tables.value = result
      const previous = selectedTable.value
      const table =
        tables.value.find((t) => t.schema === previous?.schema && t.name === previous.name) ||
        tables.value[0]
      if (table) await chooseTable(table)
      else {
        selectedTable.value = null
        data.value = null
      }
    } catch (cause) {
      error.value = message(cause)
    }
  }
  async function connect(input: ConnectionInput) {
    connecting.value = true
    connectionError.value = ''
    try {
      session.value = await $fetch<Session>('/api/session', { method: 'POST', body: input })
      ++requestVersion
      controller?.abort()
      data.value = null
      selectedRow.value = null
      activeQuery.value = null
      error.value = ''
      await loadTables()
      return true
    } catch (cause) {
      connectionError.value = message(cause)
      return false
    } finally {
      connecting.value = false
    }
  }
  async function disconnect() {
    try {
      await $fetch('/api/session', { method: 'DELETE' })
      ++requestVersion
      controller?.abort()
      session.value = null
      activeQuery.value = null
      tables.value = []
      selectedTable.value = null
      data.value = null
      selectedRow.value = null
      error.value = ''
      busy.value = false
    } catch (cause) {
      error.value = message(cause)
    }
  }
  async function sortBy(column: string) {
    if (activeQuery.value) return
    descending.value = sort.value === column ? !descending.value : false
    sort.value = column
    page.value = 0
    await loadRows()
  }
  async function changePage(next: number) {
    page.value = next
    await loadRows()
  }
  async function changePageSize(size: number) {
    pageSize.value = size
    page.value = 0
    await loadRows()
  }
  async function runQuery(sql: string) {
    if (busy.value || !sql.trim()) return
    activeQuery.value = sql.trim()
    sort.value = undefined
    descending.value = false
    page.value = 0
    await loadRows()
  }
  async function browseTable() {
    if (busy.value) return
    activeQuery.value = null
    page.value = 0
    await loadRows()
  }
  onMounted(async () => {
    try {
      providers.value = await $fetch<Provider[]>('/api/providers')
    } catch {
      /* Connection action presents an actionable error. */
    }
    try {
      const restored = await $fetch<Session>('/api/session')
      if (!session.value && !connecting.value) {
        session.value = restored
        await loadTables()
      }
    } catch {
      /* No session yet, or the backend restarted. */
    }
  })
  onScopeDispose(() => controller?.abort())
  return {
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
    connect,
    disconnect,
    chooseTable,
    loadRows,
    loadTables,
    sortBy,
    changePage,
    changePageSize,
    runQuery,
    browseTable,
  }
}

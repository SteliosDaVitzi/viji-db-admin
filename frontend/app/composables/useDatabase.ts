import type {
  ConnectionInput,
  Provider,
  Session,
  TableInfo,
  TablePage,
  SavedConnection,
  Mutation,
} from '../../shared/database'

export function useDatabase() {
  const toast = useToast()
  const providers = ref<Provider[]>([])
  const session = ref<Session | null>(null)
  const databases = ref<string[]>([])
  const savedConnections = ref<SavedConnection[]>([])
  const savingEnabled = ref(false)
  const switching = ref(false)
  const tables = ref<TableInfo[]>([])
  const tablesLoading = ref(false)
  const selectedTable = ref<TableInfo | null>(null)
  const data = shallowRef<TablePage | null>(null)
  const selectedRow = ref<number | null>(null)
  const busy = ref(false)
  const connecting = ref(false)
  const connectionError = ref('')
  const error = ref('')
  for (const source of [error, connectionError])
    watch(
      source,
      (failure) => {
        if (failure) toast.show(failure, 'error')
      },
      { flush: 'sync' },
    )
  const page = ref(0)
  const pageSize = ref(50)
  const sort = ref<string | undefined>()
  const descending = ref(false)
  const activeQuery = ref<string | null>(null)
  let requestVersion = 0
  let controller: AbortController | undefined

  function message(cause: unknown, operation?: 'save') {
    const failure = cause as { data?: { message?: string }; statusCode?: number; status?: number }
    const status = failure.statusCode || failure.status
    if (status === 401) {
      session.value = null
      return 'Your connection session expired or the backend restarted. Reconnect to continue.'
    }
    if (operation === 'save' && status === 404) {
      return 'The running backend does not support editing yet. Restart the WebDB backend, then reconnect and try again.'
    }
    if (status === 413)
      return 'This request is too large. Use a database client to edit larger records.'
    return (
      failure.data?.message ||
      (status === 403
        ? 'The request was rejected. Open WebDB at its original address and try again.'
        : status && status < 500
          ? 'The request was rejected. Refresh the table and check the entered value.'
          : 'Could not reach the database service. Check that the backend is running and try again.')
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
    if (switching.value) return
    activeQuery.value = null
    selectedTable.value = table
    page.value = 0
    sort.value = undefined
    descending.value = false
    await loadRows()
  }
  async function loadTables() {
    tablesLoading.value = true
    try {
      const currentSession = session.value
      const result = await $fetch<TableInfo[]>('/api/tables')
      if (session.value !== currentSession) return
      tables.value = result
      const previous = selectedTable.value
      const table =
        tables.value.find((t) => t.schema === previous?.schema && t.name === previous.name) ||
        tables.value.find((t) => t.schema === 'public') ||
        tables.value[0]
      if (table) await chooseTable(table)
      else {
        selectedTable.value = null
        data.value = null
      }
    } catch (cause) {
      error.value = message(cause)
    } finally {
      tablesLoading.value = false
    }
  }
  function clearWorkspace() {
    ++requestVersion
    controller?.abort()
    tables.value = []
    selectedTable.value = null
    data.value = null
    selectedRow.value = null
    activeQuery.value = null
    error.value = ''
    busy.value = false
    page.value = 0
    sort.value = undefined
    descending.value = false
  }
  async function loadSavedConnections() {
    const result = await $fetch<{ enabled: boolean; connections: SavedConnection[] }>(
      '/api/connections',
    )
    savedConnections.value = result.connections
    savingEnabled.value = result.enabled
  }
  async function loadDatabases() {
    databases.value = session.value?.allDatabases
      ? await $fetch<string[]>('/api/databases')
      : session.value
        ? [session.value.database]
        : []
  }
  async function refreshExplorer() {
    let discoveryError = ''
    try {
      await loadDatabases()
    } catch (cause) {
      discoveryError = message(cause)
    }
    await loadTables()
    if (discoveryError) error.value = discoveryError
  }
  async function switchDatabase(database: string, table?: TableInfo) {
    if (switching.value || connecting.value || database === session.value?.database) return
    switching.value = true
    clearWorkspace()
    try {
      session.value = await $fetch<Session>('/api/databases', {
        method: 'POST',
        body: { database },
      })
      selectedTable.value = table || null
    } catch (cause) {
      error.value = message(cause)
    } finally {
      switching.value = false
    }
    if (!error.value) await loadTables()
  }
  async function chooseDatabaseTable(database: string, table: TableInfo) {
    if (switching.value || connecting.value) return
    if (database === session.value?.database || !session.value?.allDatabases)
      await chooseTable(table)
    else await switchDatabase(database, table)
  }
  async function connect(input: ConnectionInput | string) {
    connecting.value = true
    connectionError.value = ''
    try {
      session.value =
        typeof input === 'string'
          ? await $fetch<Session>(`/api/connections/${encodeURIComponent(input)}/connect`, {
              method: 'POST',
            })
          : await $fetch<Session>('/api/session', { method: 'POST', body: input })
      clearWorkspace()
      await refreshExplorer()
      if (typeof input !== 'string' && input.saveName) {
        try {
          await $fetch('/api/connections', { method: 'POST', body: { name: input.saveName } })
          await loadSavedConnections()
          toast.show('Connection saved securely on the server.')
        } catch (cause) {
          error.value = `Connected, but the connection could not be saved. ${message(cause)}`
        }
      }
      toast.show('Connected to database.')
      return true
    } catch (cause) {
      connectionError.value = message(cause)
      return false
    } finally {
      connecting.value = false
    }
  }
  async function deleteSavedConnection(id: string) {
    connectionError.value = ''
    try {
      await $fetch(`/api/connections/${encodeURIComponent(id)}`, { method: 'DELETE' })
      toast.show('Saved connection removed.')
      await loadSavedConnections()
    } catch (cause) {
      connectionError.value = message(cause)
    }
  }
  async function disconnect() {
    try {
      await $fetch('/api/session', { method: 'DELETE' })
      ++requestVersion
      controller?.abort()
      session.value = null
      toast.show('Disconnected.')
      databases.value = []
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
    if (!error.value) toast.show('Query completed.')
  }
  async function browseTable() {
    if (busy.value) return
    activeQuery.value = null
    page.value = 0
    await loadRows()
  }
  async function applyMutation(mutation: Mutation) {
    try {
      return await $fetch<{ affectedRows: number }>('/api/mutations', {
        method: 'POST',
        body: { confirmation: 'APPLY', mutation },
      })
    } catch (cause) {
      throw new Error(message(cause, 'save'))
    }
  }
  onMounted(async () => {
    try {
      await loadSavedConnections()
    } catch {
      /* Saving stays disabled if unavailable. */
    }
    try {
      providers.value = await $fetch<Provider[]>('/api/providers')
    } catch {
      /* Connection action presents an actionable error. */
    }
    try {
      const restored = await $fetch<Session>('/api/session')
      if (!session.value && !connecting.value) {
        session.value = restored
        await refreshExplorer()
      }
    } catch {
      /* No session yet, or the backend restarted. */
    }
  })
  onScopeDispose(() => controller?.abort())
  return {
    applyMutation,
    providers,
    databases,
    savedConnections,
    savingEnabled,
    switching,
    switchDatabase,
    chooseDatabaseTable,
    refreshExplorer,
    deleteSavedConnection,
    session,
    tables,
    tablesLoading,
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

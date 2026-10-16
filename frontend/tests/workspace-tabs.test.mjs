import assert from 'node:assert/strict'
import { test } from 'node:test'
import { effectScope, nextTick, ref, shallowRef } from 'vue'
import { useWorkspaceTabs } from '../app/composables/useWorkspaceTabs.ts'

const customers = { schema: 'public', name: 'customers', kind: 'BASE TABLE', columns: [] }
const orders = { ...customers, name: 'orders' }
function setup(t) {
  const db = {
    session: ref({ database: 'sample', allDatabases: false }),
    tables: ref([customers, orders]),
    selectedTable: ref(null),
    data: shallowRef(null),
    activeQuery: ref(null),
    selectedRow: ref(null),
    error: ref(''),
    page: ref(0),
    pageSize: ref(50),
    sort: ref(undefined),
    descending: ref(false),
    busy: ref(false),
    switching: ref(false),
    connecting: ref(false),
    loads: 0,
    async loadRows() {
      this.loads++
      this.data.value = {
        rows: [{ source: this.activeQuery.value || this.selectedTable.value.name }],
      }
    },
    async chooseTable(table) {
      this.activeQuery.value = null
      this.selectedTable.value = table
      this.page.value = 0
      this.sort.value = undefined
      this.descending.value = false
      await this.loadRows()
    },
    async runQuery(sql) {
      this.activeQuery.value = sql
      this.page.value = 0
      await this.loadRows()
    },
  }
  const scope = effectScope()
  const tabs = scope.run(() => useWorkspaceTabs(db))
  t.after(() => scope.stop())
  return { db, tabs }
}

test('tables retain independent browsing state and opening again does not duplicate or reload', async (t) => {
  const { db, tabs } = setup(t)
  await tabs.openTable(customers)
  db.page.value = 3
  db.pageSize.value = 25
  db.sort.value = 'name'
  db.descending.value = true
  tabs.rowFilter.value = 'Ada'
  tabs.tab.value = 'structure'
  const original = db.data.value
  await tabs.openTable(orders)
  assert.equal(tabs.rowFilter.value, '')
  await tabs.openTable(customers)
  await tabs.openTable(customers)
  assert.equal(tabs.tableTabs.value.length, 2)
  assert.equal(db.loads, 2)
  assert.equal(db.data.value, original)
  assert.equal(db.page.value, 3)
  assert.equal(db.pageSize.value, 25)
  assert.equal(db.sort.value, 'name')
  assert.equal(db.descending.value, true)
  assert.equal(tabs.rowFilter.value, 'Ada')
  assert.equal(tabs.tab.value, 'structure')
})

test('query results and blank query drafts are isolated from table views', async (t) => {
  const { db, tabs } = setup(t)
  await tabs.openTable(customers)
  await tabs.runQuery('SELECT 1')
  const result = db.data.value
  await tabs.backToTable()
  assert.equal(db.activeQuery.value, null)
  assert.equal(db.data.value.rows[0].source, 'customers')
  await tabs.activateQuery('query-1')
  assert.equal(db.data.value, result)
  await tabs.activateQuery('query-2')
  assert.equal(db.data.value, null)
  await tabs.openTable(orders)
  await tabs.activateQuery('query-2')
  assert.equal(db.data.value, null)
})

test('closing selects a neighbor, permits an empty workspace, and reopening fetches fresh rows', async (t) => {
  const { db, tabs } = setup(t)
  await tabs.openTable(customers)
  await tabs.openTable(orders)
  await tabs.closeTable(orders)
  assert.equal(db.selectedTable.value.name, 'customers')
  await tabs.closeTable(customers)
  assert.equal(tabs.tableTabs.value.length, 0)
  assert.equal(db.selectedTable.value, null)
  assert.equal(db.data.value, null)
  await tabs.openTable(customers)
  assert.equal(db.loads, 3)
})

test('busy operations block switching and closing; session changes clear all cached tabs', async (t) => {
  const { db, tabs } = setup(t)
  await tabs.openTable(customers)
  db.busy.value = true
  await tabs.openTable(orders)
  await tabs.closeTable(customers)
  await tabs.activateQuery('query-2')
  assert.equal(tabs.activeView.value, tabs.key(customers))
  assert.equal(tabs.tableTabs.value.length, 1)
  db.busy.value = false
  db.session.value = { database: 'other' }
  assert.equal(tabs.tableTabs.value.length, 0)
  await tabs.openTable(customers)
  assert.equal(db.loads, 2)
})

test('mutations invalidate inactive results, removed tables are pruned, and schema names disambiguate tabs', async (t) => {
  const { db, tabs } = setup(t)
  const other = { ...customers, schema: 'sales' }
  db.tables.value = [customers, orders, other]
  await tabs.openTable(customers)
  await tabs.openTable(other)
  assert.equal(tabs.tableTabs.value.length, 2)
  tabs.invalidate()
  await tabs.openTable(customers)
  assert.equal(db.loads, 3)
  db.tables.value = [customers, orders]
  await nextTick()
  assert.equal(tabs.tableTabs.value.length, 1)
})

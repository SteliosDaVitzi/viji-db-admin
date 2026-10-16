import assert from 'node:assert/strict'
import { test } from 'node:test'
import { EditorState } from '@codemirror/state'
import { CompletionContext } from '@codemirror/autocomplete'
import { sql, PostgreSQL } from '@codemirror/lang-sql'
import { databaseCompletion, quoteSqlIdentifier, sqlSnippets } from '../app/utils/sqlCompletion.ts'

const customers = {
  schema: 'public',
  name: 'customers',
  kind: 'BASE TABLE',
  columns: ['id', 'email', 'Display Name', 'a"b'],
}
const tables = [
  customers,
  {
    schema: 'sales',
    name: 'Order Details',
    kind: 'VIEW',
    columns: ['order_id', 'total'],
  },
]
function context(doc, explicit = true) {
  return new CompletionContext(
    EditorState.create({
      doc,
      extensions: [sql({ dialect: PostgreSQL })],
    }),
    doc.length,
    explicit,
  )
}
const complete = (doc, source = databaseCompletion(tables, customers)) => source(context(doc))

test('suggests tables, schemas, selected fields and qualified fields from other schemas', async () => {
  const top = await complete('SELECT ')
  assert.equal(top.options.find((o) => o.label === 'customers').apply, '"customers"')
  assert.equal(top.options.find((o) => o.label === 'email').apply, '"email"')
  assert.equal(top.options.find((o) => o.label === 'sales').apply, '"sales"')
  assert.equal(top.options.find((o) => o.label === 'total').detail, 'sales.Order Details')
  const qualified = await complete('SELECT sales."Order Details".')
  assert.equal(qualified.options.find((o) => o.label === 'total').apply, '"total"')
})

test('resolves table aliases to their columns', async () => {
  const result = await complete('SELECT * FROM sales."Order Details" AS o WHERE o.')
  assert.deepEqual(
    result.options.map((o) => o.label),
    ['order_id', 'total'],
  )
})

test('quotes spaces, reserved names and embedded quotes safely', async () => {
  assert.equal(quoteSqlIdentifier('select'), '"select"')
  assert.equal(quoteSqlIdentifier('a"b'), '"a""b"')
  const result = await complete('SELECT "')
  assert.equal(result.options.find((o) => o.label === '"a""b"').apply, '"a""b"')
  assert.equal(result.options.find((o) => o.label === '"Display Name"').apply, '"Display Name"')
})

test('keeps duplicate table names separate across schemas', async () => {
  const source = databaseCompletion(
    [customers, { ...customers, schema: 'private', columns: ['secret'] }],
    customers,
  )
  const result = await complete('SELECT private.customers.', source)
  assert.deepEqual(
    result.options.map((o) => o.label),
    ['secret'],
  )
})

test('does not suggest database names or snippets in strings and comments', async () => {
  for (const doc of ["SELECT 'hello", '-- customer', '/* customer']) {
    assert.equal(await complete(doc), null)
    assert.equal(await sqlSnippets(context(doc)), null)
  }
})

test('offers common SQL expressions with editable snippet placeholders', async () => {
  const result = await sqlSnippets(context('SEL'))
  for (const label of ['SELECT FROM WHERE', 'COUNT', 'COALESCE', 'LEFT JOIN', 'GROUP BY']) {
    assert.equal(typeof result.options.find((o) => o.label === label).apply, 'function')
  }
})

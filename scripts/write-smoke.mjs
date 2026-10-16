import assert from 'node:assert/strict'
import { randomUUID } from 'node:crypto'

// Never infer a database target from a user's active application connection.
assert.ok(
  process.env.WEBDB_TEST_CONNECTION_JSON,
  'Configure WEBDB_TEST_CONNECTION_JSON for a disposable database.',
)
const connection = JSON.parse(process.env.WEBDB_TEST_CONNECTION_JSON)
assert.equal(connection.provider, 'postgres')
const base = process.env.WEBDB_URL || 'http://127.0.0.1:34616'
const connected = await fetch(`${base}/api/session`, {
  method: 'POST',
  headers: { 'content-type': 'application/json', origin: base },
  body: JSON.stringify(connection),
})
assert.equal(connected.status, 200)
const cookie = connected.headers.get('set-cookie')?.split(';')[0]
assert.ok(cookie)
const headers = { cookie, 'content-type': 'application/json', origin: base }
async function mutate(mutation, confirmation = 'APPLY') {
  return fetch(`${base}/api/mutations`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ confirmation, mutation }),
  })
}
const schema = `smoke_${randomUUID().replaceAll('-', '')}`
const qualified = `"${schema}"."records"`
assert.equal((await mutate({ action: 'execute', sql: `CREATE SCHEMA "${schema}"` })).status, 200)
try {
  assert.equal(
    (
      await mutate({
        action: 'execute',
        sql: `CREATE TABLE ${qualified} (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY, note text NOT NULL, amount numeric(20,4) CHECK(amount>0))`,
      })
    ).status,
    200,
  )
  const insert = {
    action: 'insert',
    schema,
    table: 'records',
    values: { note: 'original', amount: '1234567890.1234' },
  }
  assert.equal((await mutate(insert, '')).status, 400)
  assert.equal((await mutate(insert)).status, 200)
  async function browse() {
    const response = await fetch(`${base}/api/rows?schema=${schema}&table=records`, { headers })
    assert.equal(response.status, 200)
    return response.json()
  }
  const page = await browse()
  assert.equal(page.columns.find((c) => c.name === 'id').readOnly, true)
  assert.equal(page.rows[0].amount, '1234567890.1234')
  const update = {
    action: 'update',
    schema,
    table: 'records',
    original: page.rows[0],
    values: { note: 'edited' },
  }
  assert.equal((await mutate(update)).status, 200)
  assert.equal((await mutate(update)).status, 400)
  const current = (await browse()).rows[0]
  assert.equal(current.note, 'edited')
  const rejected = await mutate({ ...update, original: current, values: { amount: '-1' } })
  assert.equal(rejected.status, 400)
  assert.match((await rejected.json()).message, /Check constraint/)
  assert.equal(
    (await mutate({ action: 'delete', schema, table: 'records', original: current })).status,
    200,
  )
  assert.equal((await browse()).rows.length, 0)
  assert.equal((await mutate({ action: 'dropTable', schema, table: 'records' })).status, 200)
  console.log(
    'Write smoke checks passed: proxy, confirmation, generated fields, exact values, update, stale detection, constraint failure, delete, and drop table.',
  )
} finally {
  await mutate({ action: 'execute', sql: `DROP SCHEMA "${schema}" CASCADE` })
  await fetch(`${base}/api/session`, { method: 'DELETE', headers })
}

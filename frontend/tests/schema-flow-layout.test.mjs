import assert from 'node:assert/strict'
import test from 'node:test'
import { layoutSchemaTables } from '../app/utils/schemaFlowLayout.ts'

function table(name, foreignKeys = []) {
  return {
    schema: 'public',
    name,
    kind: 'BASE TABLE',
    columns: ['id', 'parent_id'],
    primaryKeys: ['id'],
    foreignKeys,
  }
}

function foreignKey(name, column, referencedTable, referencedColumn = 'id') {
  return {
    name,
    column,
    referencedSchema: 'public',
    referencedTable,
    referencedColumn,
  }
}

function point(positions, name) {
  return positions[JSON.stringify(['public', name])]
}

test('places referenced tables above dependent tables', () => {
  const positions = layoutSchemaTables([
    table('line_items', [foreignKey('line_items_order_id_fkey', 'parent_id', 'orders')]),
    table('orders', [foreignKey('orders_customer_id_fkey', 'parent_id', 'customers')]),
    table('customers'),
  ])

  assert.ok(point(positions, 'customers').y < point(positions, 'orders').y)
  assert.ok(point(positions, 'orders').y < point(positions, 'line_items').y)
})

test('keeps converging source tables in the same source row without overlap', () => {
  const positions = layoutSchemaTables([
    table('orders', [
      foreignKey('orders_customer_id_fkey', 'parent_id', 'customers'),
      foreignKey('orders_product_id_fkey', 'id', 'products'),
    ]),
    table('customers'),
    table('products'),
  ])
  const customer = point(positions, 'customers')
  const product = point(positions, 'products')

  assert.equal(customer.y, product.y)
  assert.notEqual(customer.x, product.x)
  assert.ok(point(positions, 'orders').y > customer.y)
})

test('groups a cyclic relationship in one row and lays its dependents below', () => {
  const positions = layoutSchemaTables([
    table('alpha', [foreignKey('alpha_beta_fkey', 'parent_id', 'beta')]),
    table('beta', [foreignKey('beta_alpha_fkey', 'parent_id', 'alpha')]),
    table('gamma', [foreignKey('gamma_beta_fkey', 'parent_id', 'beta')]),
  ])

  assert.equal(point(positions, 'alpha').y, point(positions, 'beta').y)
  assert.notEqual(point(positions, 'alpha').x, point(positions, 'beta').x)
  assert.ok(point(positions, 'gamma').y > point(positions, 'beta').y)
})

test('keeps independent flows in separate columns and packs unrelated tables below', () => {
  const positions = layoutSchemaTables([
    table('billing_child', [foreignKey('billing_fk', 'parent_id', 'billing_parent')]),
    table('billing_parent'),
    table('catalog_child', [foreignKey('catalog_fk', 'parent_id', 'catalog_parent')]),
    table('catalog_parent'),
    table('orphan_a'),
    table('orphan_b'),
  ])
  const billingParent = point(positions, 'billing_parent')
  const catalogParent = point(positions, 'catalog_parent')

  assert.equal(billingParent.y, catalogParent.y)
  assert.notEqual(billingParent.x, catalogParent.x)
  assert.ok(point(positions, 'orphan_a').y > point(positions, 'catalog_child').y)
  assert.notEqual(point(positions, 'orphan_a').x, point(positions, 'orphan_b').x)
})

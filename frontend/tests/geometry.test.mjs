import assert from 'node:assert/strict'
import { test } from 'node:test'
import { isSpatialType } from '../app/utils/geometry.ts'

test('only spatial scalar columns offer a map, including constrained types', () => {
  for (const type of ['geometry', 'geography', 'GEOMETRY', 'geometry(Point,4326)', ' geography (PolygonZ, 2100) ']) {
    assert.equal(isSpatialType(type), true, type)
  }
  for (const type of ['json', 'text', 'point', 'geometry[]', 'geography[]', 'USER-DEFINED', 'mygeometry']) {
    assert.equal(isSpatialType(type), false, type)
  }
})

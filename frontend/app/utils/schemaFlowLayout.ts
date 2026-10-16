import type { TableInfo } from '../../shared/database'

export interface DiagramPoint {
  x: number
  y: number
}

const nodeWidth = 292
const layerGap = 156
const nodeGap = 26
const chartGap = 112
const padding = 72
const columnHeight = 30
const shownColumns = 8

function keyOf(table: Pick<TableInfo, 'schema' | 'name'>) {
  return JSON.stringify([table.schema, table.name])
}

function compareTables(a: TableInfo, b: TableInfo) {
  return a.schema.localeCompare(b.schema) || a.name.localeCompare(b.name)
}

function tableHeight(table: TableInfo) {
  const visible = Math.min(table.columns.length, shownColumns)
  const fieldsHeight = table.columns.length
    ? 10 + visible * columnHeight + (table.columns.length > shownColumns ? 30 : 0)
    : 41
  return 56 + fieldsHeight + 39
}

function stronglyConnectedComponents(keys: string[], children: Map<string, Set<string>>) {
  let index = 0
  const indices = new Map<string, number>()
  const lowLinks = new Map<string, number>()
  const stack: string[] = []
  const onStack = new Set<string>()
  const components: string[][] = []

  function visit(key: string) {
    indices.set(key, index)
    lowLinks.set(key, index)
    index += 1
    stack.push(key)
    onStack.add(key)

    for (const child of children.get(key) ?? []) {
      if (!indices.has(child)) {
        visit(child)
        lowLinks.set(key, Math.min(lowLinks.get(key)!, lowLinks.get(child)!))
      } else if (onStack.has(child)) {
        lowLinks.set(key, Math.min(lowLinks.get(key)!, indices.get(child)!))
      }
    }

    if (lowLinks.get(key) !== indices.get(key)) return
    const component: string[] = []
    let member: string
    do {
      member = stack.pop()!
      onStack.delete(member)
      component.push(member)
    } while (member !== key)
    components.push(component)
  }

  for (const key of keys) {
    if (!indices.has(key)) visit(key)
  }
  return components
}

function componentLayout(
  componentKeys: string[],
  tablesByKey: Map<string, TableInfo>,
  children: Map<string, Set<string>>,
  startX: number,
  startY: number,
) {
  const stronglyConnected = stronglyConnectedComponents(componentKeys, children)
  const componentOf = new Map<string, number>()
  stronglyConnected.forEach((members, id) => members.forEach((key) => componentOf.set(key, id)))
  const upstream = stronglyConnected.map(() => new Set<number>())
  const downstream = stronglyConnected.map(() => new Set<number>())

  for (const key of componentKeys) {
    const source = componentOf.get(key)!
    for (const child of children.get(key) ?? []) {
      const target = componentOf.get(child)!
      if (source === target) continue
      downstream[source]!.add(target)
      upstream[target]!.add(source)
    }
  }

  const rank = stronglyConnected.map(() => 0)
  const indegree = upstream.map((links) => links.size)
  const label = (id: number) =>
    stronglyConnected[id]!.map((key) => tablesByKey.get(key)!).sort(compareTables)[0]!
  const queue = indegree
    .map((count, id) => (count === 0 ? id : -1))
    .filter((id) => id >= 0)
    .sort((a, b) => compareTables(label(a), label(b)))

  while (queue.length) {
    const source = queue.shift()!
    for (const target of downstream[source]!) {
      rank[target] = Math.max(rank[target]!, rank[source]! + 1)
      indegree[target] = indegree[target]! - 1
      if (indegree[target] === 0) {
        queue.push(target)
        queue.sort((a, b) => compareTables(label(a), label(b)))
      }
    }
  }

  const maxRank = Math.max(...rank)
  const layers = Array.from({ length: maxRank + 1 }, () => [] as number[])
  rank.forEach((layer, id) => layers[layer]!.push(id))
  for (const layer of layers) layer.sort((a, b) => compareTables(label(a), label(b)))

  function barycenter(blockId: number, direction: 'upstream' | 'downstream', order: number[][]) {
    const neighboringBlocks = direction === 'upstream' ? upstream[blockId]! : downstream[blockId]!
    const scores: number[] = []
    for (const neighbor of neighboringBlocks) {
      const neighborRank = rank[neighbor]!
      if (
        direction === 'upstream' ? neighborRank >= rank[blockId]! : neighborRank <= rank[blockId]!
      )
        continue
      const indexInLayer = order[neighborRank]!.indexOf(neighbor)
      scores.push((indexInLayer + 0.5) / order[neighborRank]!.length)
    }
    return scores.length ? scores.reduce((sum, score) => sum + score, 0) / scores.length : null
  }

  function sortLayer(layerRank: number, direction: 'upstream' | 'downstream') {
    const layer = layers[layerRank]!
    const scores = new Map(layer.map((id) => [id, barycenter(id, direction, layers)]))
    layer.sort((a, b) => {
      const scoreA = scores.get(a)
      const scoreB = scores.get(b)
      if (scoreA !== null && scoreA !== undefined && scoreB !== null && scoreB !== undefined) {
        if (scoreA !== scoreB) return scoreA - scoreB
      } else if (scoreA !== null && scoreA !== undefined) return -1
      else if (scoreB !== null && scoreB !== undefined) return 1
      return compareTables(label(a), label(b))
    })
  }

  for (let pass = 0; pass < 3; pass += 1) {
    for (let layer = 1; layer <= maxRank; layer += 1) sortLayer(layer, 'upstream')
    for (let layer = maxRank - 1; layer >= 0; layer -= 1) sortLayer(layer, 'downstream')
  }

  const layerHeights = layers.map((layer) =>
    Math.max(
      ...layer.flatMap((id) =>
        stronglyConnected[id]!.map((key) => tableHeight(tablesByKey.get(key)!)),
      ),
    ),
  )
  const layerWidths = layers.map((layer) => {
    const tableCount = layer.reduce((count, id) => count + stronglyConnected[id]!.length, 0)
    return tableCount * nodeWidth + Math.max(0, tableCount - 1) * nodeGap
  })
  const height =
    layerHeights.reduce((total, layerHeight) => total + layerHeight, 0) +
    Math.max(0, layers.length - 1) * layerGap
  const width = Math.max(...layerWidths)
  const positions: Record<string, DiagramPoint> = {}
  let currentY = startY

  layers.forEach((layer, layerRank) => {
    const layerWidth = layerWidths[layerRank]!
    let x = startX + (width - layerWidth) / 2
    for (const blockId of layer) {
      const members = stronglyConnected[blockId]!.map((key) => tablesByKey.get(key)!).sort(
        compareTables,
      )
      for (const table of members) {
        positions[keyOf(table)] = {
          x,
          y: currentY,
        }
        x += nodeWidth + nodeGap
      }
    }
    currentY += layerHeights[layerRank]! + (layerRank < maxRank ? layerGap : 0)
  })

  return { positions, width, height }
}

export function layoutSchemaTables(tables: TableInfo[]): Record<string, DiagramPoint> {
  const orderedTables = [...tables].sort(compareTables)
  const tablesByKey = new Map(orderedTables.map((table) => [keyOf(table), table]))
  const children = new Map(orderedTables.map((table) => [keyOf(table), new Set<string>()]))
  const neighbors = new Map(orderedTables.map((table) => [keyOf(table), new Set<string>()]))
  const relationalKeys = new Set<string>()

  for (const dependent of orderedTables) {
    const dependentKey = keyOf(dependent)
    for (const foreignKey of dependent.foreignKeys ?? []) {
      const parentKey = keyOf({
        schema: foreignKey.referencedSchema,
        name: foreignKey.referencedTable,
      })
      if (!tablesByKey.has(parentKey)) continue
      children.get(parentKey)!.add(dependentKey)
      neighbors.get(dependentKey)!.add(parentKey)
      neighbors.get(parentKey)!.add(dependentKey)
      relationalKeys.add(dependentKey)
      relationalKeys.add(parentKey)
    }
  }

  const visited = new Set<string>()
  const components: string[][] = []
  for (const table of orderedTables) {
    const start = keyOf(table)
    if (visited.has(start)) continue
    const component: string[] = []
    const queue = [start]
    visited.add(start)
    while (queue.length) {
      const key = queue.shift()!
      component.push(key)
      for (const neighbor of neighbors.get(key) ?? []) {
        if (visited.has(neighbor)) continue
        visited.add(neighbor)
        queue.push(neighbor)
      }
    }
    components.push(component)
  }

  const relatedComponents = components
    .filter((component) => component.some((key) => relationalKeys.has(key)))
    .sort((a, b) => compareTables(tablesByKey.get(a[0]!)!, tablesByKey.get(b[0]!)!))
  const independentTables = orderedTables.filter((table) => !relationalKeys.has(keyOf(table)))
  const positions: Record<string, DiagramPoint> = {}
  let currentX = padding
  let flowHeight = 0

  for (const component of relatedComponents) {
    const layout = componentLayout(component, tablesByKey, children, currentX, padding)
    Object.assign(positions, layout.positions)
    currentX += layout.width + chartGap
    flowHeight = Math.max(flowHeight, layout.height)
  }

  if (independentTables.length) {
    const columns = Math.min(5, Math.ceil(Math.sqrt(independentTables.length)))
    const rows = Math.ceil(independentTables.length / columns)
    const rowHeight = Math.max(...independentTables.map(tableHeight))
    const startY = relatedComponents.length ? padding + flowHeight + chartGap : padding
    independentTables.forEach((table, index) => {
      positions[keyOf(table)] = {
        x: padding + (index % columns) * (nodeWidth + nodeGap),
        y: startY + Math.floor(index / columns) * (rowHeight + nodeGap),
      }
    })
    flowHeight += rows * rowHeight + Math.max(0, rows - 1) * nodeGap
  }

  return positions
}

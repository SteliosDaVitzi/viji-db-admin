import { completeFromList, ifNotIn, snippetCompletion } from '@codemirror/autocomplete'
import type { Completion, CompletionSource } from '@codemirror/autocomplete'
import { schemaCompletionSource } from '@codemirror/lang-sql'
import { providerOptions, quoteIdentifier } from '../providers/index.ts'
import type { SQLNamespace } from '@codemirror/lang-sql'
import type { TableInfo } from '../../shared/database'

export const quoteSqlIdentifier = quoteIdentifier

export function databaseCompletion(
  tables: TableInfo[],
  selected: TableInfo | null,
  provider = 'postgres',
): CompletionSource {
  const quote = (name: string) => quoteIdentifier(name, provider)
  const schema: Record<string, SQLNamespace> = Object.create(null)
  const fields = new Map<string, Completion>()
  for (const table of tables) {
    for (const name of table.columns ?? []) {
      if (!fields.has(name)) {
        fields.set(name, {
          label: name,
          type: 'property',
          apply: quote(name),
          detail: `${table.schema}.${table.name}`,
        })
      }
    }
    const namespace = (schema[table.schema] ??= {
      self: { label: table.schema, type: 'namespace', apply: quote(table.schema) },
      children: Object.create(null) as Record<string, SQLNamespace>,
    }) as { self: Completion; children: Record<string, SQLNamespace> }
    namespace.children[table.name] = {
      self: { label: table.name, type: 'type', apply: quote(table.name) },
      children: (table.columns ?? []).map((name) => ({
        label: name,
        type: 'property',
        apply: quote(name),
      })),
    }
  }
  const source = schemaCompletionSource({
    dialect: providerOptions(provider).dialect,
    schema,
    defaultSchema:
      selected?.schema ??
      (provider === 'sqlserver' ? 'dbo' : provider === 'postgres' ? 'public' : undefined),
    defaultTable: selected?.name,
  })
  return ifNotIn(['String', 'LineComment', 'BlockComment'], async (context) => {
    const result = await source(context)
    if (!result) return null
    // Offer known fields before FROM has been written, when no table can be inferred yet.
    const qualified = /\.\s*$/.test(context.state.sliceDoc(0, result.from))
    const quoted =
      context.state.sliceDoc(result.from, result.from + 1) ===
      (provider === 'mysql' || provider === 'mariadb' ? '`' : provider === 'sqlserver' ? '[' : '"')
    if (!qualified && !quoted) {
      const existing = new Set(result.options.map((option) => option.label))
      return {
        ...result,
        options: [
          ...result.options,
          ...[...fields.values()].filter((field) => !existing.has(field.label)),
        ],
      }
    }
    // CodeMirror wraps labels inside a quoted identifier. Escape embedded quotes too.
    if (quoted) {
      return {
        ...result,
        options: result.options.map((option) => {
          const name = option.label.replace(/^["`\[]|["`\]]$/g, '')
          return { ...option, label: quote(name), apply: quote(name) }
        }),
      }
    }
    return result
  })
}

export function providerSnippets(provider = 'postgres') {
  return ifNotIn(
    ['String', 'LineComment', 'BlockComment', 'QuotedIdentifier', '.', 'CompositeIdentifier'],
    completeFromList([
      snippetCompletion('SELECT ${columns}\nFROM ${table}\nWHERE ${condition};', {
        label: 'SELECT FROM WHERE',
        detail: 'Filtered query',
        type: 'text',
      }),
      snippetCompletion('COUNT(${*})', { label: 'COUNT', detail: 'Count rows', type: 'function' }),
      snippetCompletion('SUM(${column})', { label: 'SUM', detail: 'Sum values', type: 'function' }),
      snippetCompletion('AVG(${column})', {
        label: 'AVG',
        detail: 'Average values',
        type: 'function',
      }),
      snippetCompletion('COALESCE(${column}, ${fallback})', {
        label: 'COALESCE',
        detail: 'Replace null values',
        type: 'function',
      }),
      snippetCompletion('CASE WHEN ${condition} THEN ${value} ELSE ${fallback} END', {
        label: 'CASE WHEN',
        detail: 'Conditional expression',
        type: 'text',
      }),
      snippetCompletion('LEFT JOIN ${table} ON ${condition}', {
        label: 'LEFT JOIN',
        detail: 'Join matching rows',
        type: 'text',
      }),
      snippetCompletion('INNER JOIN ${table} ON ${condition}', {
        label: 'INNER JOIN',
        detail: 'Join matching rows',
        type: 'text',
      }),
      snippetCompletion('GROUP BY ${column}', { label: 'GROUP BY', type: 'text' }),
      snippetCompletion('ORDER BY ${column} ${DESC}', { label: 'ORDER BY', type: 'text' }),
      provider === 'sqlserver'
        ? snippetCompletion('TOP (${100})', { label: 'TOP', type: 'text' })
        : snippetCompletion('LIMIT ${100}', { label: 'LIMIT', type: 'text' }),
      snippetCompletion('IS NOT NULL', { label: 'IS NOT NULL', type: 'text' }),
    ]),
  )
}
export const sqlSnippets = providerSnippets()

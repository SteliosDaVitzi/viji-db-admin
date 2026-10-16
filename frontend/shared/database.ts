export interface Provider {
  id: string
  name: string
}
export interface ConnectionInput {
  provider: string
  host: string
  port: number
  database: string
  username: string
  password: string
  ssl: boolean
  allDatabases?: boolean
  saveName?: string
}
export interface SavedConnection {
  id: string
  name: string
}
export interface Session {
  provider: string
  database: string
  host: string
  expiresIn: number
  allDatabases: boolean
}
export interface TableInfo {
  schema: string
  name: string
  kind: string
}
export interface ColumnInfo {
  name: string
  dataType: string
  nullable: boolean
  primaryKey: boolean
  readOnly: boolean
  defaultValue: string | null
}
export interface TablePage {
  columns: ColumnInfo[]
  rows: Record<string, unknown>[]
  hasMore: boolean
  page: number
  pageSize: number
}

export type Mutation =
  | { action: 'insert'; schema: string; table: string; values: Record<string, string | null> }
  | {
      action: 'update'
      schema: string
      table: string
      original: Record<string, unknown>
      values: Record<string, string | null>
    }
  | { action: 'delete'; schema: string; table: string; original: Record<string, unknown> }
  | { action: 'createDatabase' | 'dropDatabase'; name: string }
  | { action: 'dropTable'; schema: string; table: string }
  | { action: 'execute'; sql: string }

export interface DatabaseSettings {
  provider: string
  database: string
  properties: Record<string, unknown>
  settings: DatabaseSetting[]
  canEdit: boolean
}
export interface DatabaseSetting {
  name: string
  setting: string | null
  databaseValue: string | null
  editable: boolean
  databaseScoped: boolean
  unit: string | null
  category: string
  short_desc: string
  context: string
  vartype: string
  min_val: string | null
  max_val: string | null
  enumvals: string[] | null
  pending_restart: boolean
  source: string
}
export interface ContextAction {
  label: string
  disabled?: boolean
  run: () => void | Promise<void>
}

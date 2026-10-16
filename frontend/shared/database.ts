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
}
export interface Session {
  provider: string
  database: string
  host: string
  expiresIn: number
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
}
export interface TablePage {
  columns: ColumnInfo[]
  rows: Record<string, unknown>[]
  hasMore: boolean
  page: number
  pageSize: number
}

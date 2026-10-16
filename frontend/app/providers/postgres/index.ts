import { PostgreSQL } from '@codemirror/lang-sql'
export default {
  editing: true,
  nameLimit: 63,
  recordHelp: 'Values use PostgreSQL text input, including JSON and arrays.',
  ddlWarning: 'Changes commit immediately after confirmation.',
  port: 5432,
  username: 'postgres',
  maintenance: 'through the postgres maintenance database',
  dialect: PostgreSQL,
}

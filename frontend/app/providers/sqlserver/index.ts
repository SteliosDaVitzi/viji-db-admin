import { MSSQL } from '@codemirror/lang-sql'
export default {
  editing: true,
  nameLimit: 128,
  recordHelp:
    'Values use SQL Server text input. Binary fields use hexadecimal text without a 0x prefix; booleans use 0 or 1.',
  ddlWarning: 'Changes commit immediately after confirmation.',
  port: 1433,
  username: 'sa',
  maintenance: 'through the master database',
  dialect: MSSQL,
}

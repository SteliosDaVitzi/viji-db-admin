import { MySQL } from '@codemirror/lang-sql'
export default {
  editing: true,
  nameLimit: 64,
  recordHelp:
    'Record editing requires InnoDB. Values use MySQL text input. Binary and BIT fields use hexadecimal text without a 0x prefix.',
  ddlWarning:
    'MySQL and MariaDB schema changes implicitly commit and cannot be rolled back. Raw SQL follows the storage engine transaction rules.',
  port: 3306,
  username: 'root',
  maintenance: 'without a default database',
  dialect: MySQL,
}

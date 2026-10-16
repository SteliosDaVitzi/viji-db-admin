<script setup lang="ts">
import { providerOptions, quoteIdentifier } from '../providers'
import type { Mutation, TableInfo } from '../../shared/database'

interface NewField {
  name: string
  dataType: string
  nullable: boolean
  primaryKey: boolean
  unique: boolean
}

const props = defineProps<{
  provider: string
  database: string
  table: TableInfo | null
  apply: (mutation: Mutation) => Promise<{ affectedRows: number }>
}>()
const emit = defineEmits<{ close: []; saved: [] }>()
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const options = computed(() => providerOptions(props.provider))
const typeSuggestions = computed(() => {
  const common = ['smallint', 'integer', 'bigint', 'varchar(255)', 'text', 'decimal(10,2)', 'date']
  if (props.provider === 'sqlserver')
    return [...common, 'bit', 'nvarchar(255)', 'datetime2', 'uniqueidentifier', 'varbinary(max)']
  if (props.provider === 'mysql' || props.provider === 'mariadb')
    return [...common, 'tinyint', 'boolean', 'datetime', 'json', 'blob']
  return [...common, 'boolean', 'timestamp with time zone', 'uuid', 'jsonb', 'bytea']
})
const defaultSchema =
  props.provider === 'mysql' || props.provider === 'mariadb'
    ? props.database
    : props.provider === 'sqlserver'
      ? 'dbo'
      : 'public'
const schema = ref(props.table?.schema ?? defaultSchema)
const tableName = ref(props.table?.name ?? 'new_table')
const identityType =
  props.provider === 'sqlserver'
    ? 'bigint IDENTITY(1,1)'
    : props.provider === 'postgres'
      ? 'bigint GENERATED ALWAYS AS IDENTITY'
      : 'bigint AUTO_INCREMENT'
const fields = ref<NewField[]>(
  props.table
    ? []
    : [
        { name: 'id', dataType: identityType, nullable: false, primaryKey: true, unique: false },
        {
          name: 'name',
          dataType: 'varchar(255)',
          nullable: false,
          primaryKey: false,
          unique: false,
        },
      ],
)
const saving = ref(false)
const error = ref('')
useActionFeedback(error)
const target = computed(
  () =>
    `${quoteIdentifier(schema.value, props.provider)}.${quoteIdentifier(tableName.value, props.provider)}`,
)
const canAddPrimaryKey = computed(() => !props.table?.primaryKeys?.length)

function validIdentifier(value: string) {
  const trimmed = value.trim()
  return (
    !!trimmed &&
    new TextEncoder().encode(trimmed).length <= options.value.nameLimit &&
    !/[\u0000-\u001f\u007f]/.test(trimmed)
  )
}
const builtInTypes = new Set([
  'smallint',
  'int',
  'integer',
  'bigint',
  'tinyint',
  'mediumint',
  'serial',
  'smallserial',
  'bigserial',
  'real',
  'float',
  'double',
  'double precision',
  'numeric',
  'decimal',
  'number',
  'money',
  'smallmoney',
  'char',
  'nchar',
  'varchar',
  'nvarchar',
  'character',
  'character varying',
  'text',
  'tinytext',
  'mediumtext',
  'longtext',
  'ntext',
  'binary',
  'varbinary',
  'blob',
  'tinyblob',
  'mediumblob',
  'longblob',
  'bytea',
  'bool',
  'boolean',
  'bit',
  'date',
  'time',
  'timetz',
  'timestamp',
  'timestamptz',
  'datetime',
  'datetime2',
  'datetimeoffset',
  'smalldatetime',
  'year',
  'uuid',
  'uniqueidentifier',
  'json',
  'jsonb',
])
function validDataType(value: string) {
  const trimmed = value.trim()
  if (trimmed.toLowerCase() === identityType.toLowerCase()) return true
  const match = trimmed.match(
    /^([a-z]+(?:\s+[a-z]+)*)(?:\(\s*(?:max|\d+(?:\s*,\s*\d+)?)\s*\))?(?:\s+(?:unsigned|with time zone|without time zone))?$/i,
  )
  return !!match && builtInTypes.has(match[1]!.toLowerCase().replace(/\s+/g, ' '))
}
const fieldsValid = computed(() => {
  if (!fields.value.length || fields.value.length > 50) return false
  const names = fields.value.map((field) => field.name.trim().toLowerCase())
  if (new Set(names).size !== names.length) return false
  if (
    names.some((name) => props.table?.columns.some((existing) => existing.toLowerCase() === name))
  )
    return false
  if (fields.value.filter((field) => field.primaryKey).length > 1) return false
  return fields.value.every(
    (field) =>
      validIdentifier(field.name) && validDataType(field.dataType) && field.dataType.length <= 128,
  )
})
const canSave = computed(
  () =>
    !saving.value &&
    fieldsValid.value &&
    (props.table !== null || (validIdentifier(schema.value) && validIdentifier(tableName.value))),
)

function addField() {
  if (fields.value.length >= 50) return
  fields.value.push({
    name: `new_field_${fields.value.length + 1}`,
    dataType: 'varchar(255)',
    nullable: true,
    primaryKey: false,
    unique: false,
  })
}
function removeField(index: number) {
  if (saving.value) return
  fields.value.splice(index, 1)
}
function columnDefinition(field: NewField) {
  const nullability = field.primaryKey ? ' NOT NULL' : field.nullable ? ' NULL' : ' NOT NULL'
  return `${quoteIdentifier(field.name.trim(), props.provider)} ${field.dataType.trim()}${nullability}${field.primaryKey ? ' PRIMARY KEY' : field.unique ? ' UNIQUE' : ''}`
}
function statement() {
  if (!props.table) {
    return `CREATE TABLE ${target.value} (\n  ${fields.value.map(columnDefinition).join(',\n  ')}\n);`
  }
  const definitions = fields.value.map(columnDefinition)
  if (props.provider === 'sqlserver')
    return `ALTER TABLE ${target.value}\n  ADD ${definitions.join(',\n  ')};`
  return `ALTER TABLE ${target.value}\n  ${definitions.map((definition) => `ADD COLUMN ${definition}`).join(',\n  ')};`
}
async function save() {
  if (!canSave.value) return
  saving.value = true
  error.value = ''
  try {
    await props.apply({ action: 'execute', sql: statement() })
    emit('saved')
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : 'Could not apply the table changes.'
  } finally {
    saving.value = false
  }
}
onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog
    ref="dialog"
    class="mutation-dialog schema-table-editor"
    :aria-label="table ? 'Edit table details' : 'Create table'"
    @cancel="saving && $event.preventDefault()"
    @close="emit('close')"
  >
    <form @submit.prevent="save">
      <div class="inspector-heading">
        <h2>{{ table ? 'Edit table details' : 'Create table' }}</h2>
        <button
          type="button"
          class="icon-button"
          aria-label="Close editor"
          :disabled="saving"
          @click="dialog?.close()"
        >
          <AppIcon name="close" :size="18" />
        </button>
      </div>
      <p class="muted">
        {{ database }}<template v-if="table"> / {{ table.schema }}.{{ table.name }}</template>
      </p>
      <template v-if="!table">
        <label
          >Schema<input
            v-model="schema"
            required
            :maxlength="options.nameLimit"
            autocomplete="off"
            spellcheck="false"
        /></label>
        <label
          >Table name<input
            v-model="tableName"
            required
            :maxlength="options.nameLimit"
            autocomplete="off"
            spellcheck="false"
        /></label>
      </template>
      <section v-else class="schema-existing-fields" aria-label="Existing fields">
        <h3>Existing fields</h3>
        <div v-for="column in table.columns" :key="column" class="schema-existing-field">
          <AppIcon :name="table.primaryKeys?.includes(column) ? 'key' : 'table'" :size="14" />
          <span>{{ column }}</span>
          <small v-if="table.primaryKeys?.includes(column)">PK</small>
          <small v-else-if="table.foreignKeys?.some((key) => key.column === column)">FK</small>
        </div>
      </section>
      <section class="schema-new-fields" aria-label="Fields to add">
        <div class="schema-fields-heading">
          <h3>{{ table ? 'Add fields' : 'Fields' }}</h3>
          <button
            type="button"
            class="button secondary small"
            :disabled="saving || fields.length >= 50"
            @click="addField"
          >
            <AppIcon name="plus" :size="15" /> Add field
          </button>
        </div>
        <p v-if="!fields.length" class="muted">Add a field to continue.</p>
        <p
          v-if="fields.some((field) => field.dataType && !validDataType(field.dataType))"
          class="schema-field-validation"
          role="status"
        >
          Choose a supported database type such as integer, varchar(255), or decimal(10,2).
        </p>
        <div v-for="(field, index) in fields" :key="index" class="schema-new-field">
          <label
            >Field name<input
              v-model="field.name"
              required
              :maxlength="options.nameLimit"
              :disabled="saving"
              autocomplete="off"
              spellcheck="false"
          /></label>
          <label
            >Type<input
              v-model="field.dataType"
              required
              maxlength="128"
              :disabled="saving"
              placeholder="varchar(255)"
              list="schema-field-type-options"
              autocomplete="off"
              spellcheck="false"
          /></label>
          <label class="schema-field-check">
            <input
              v-model="field.nullable"
              type="checkbox"
              :disabled="saving || field.primaryKey"
            />
            Nullable
          </label>
          <label v-if="canAddPrimaryKey" class="schema-field-check">
            <input
              v-model="field.primaryKey"
              type="checkbox"
              :disabled="
                saving || fields.some((item, itemIndex) => itemIndex !== index && item.primaryKey)
              "
            />
            Primary key
          </label>
          <label class="schema-field-check">
            <input v-model="field.unique" type="checkbox" :disabled="saving || field.primaryKey" />
            Unique
          </label>
          <button
            type="button"
            class="icon-button schema-field-remove"
            :aria-label="`Remove field ${index + 1}`"
            title="Remove field"
            :disabled="saving"
            @click="removeField(index)"
          >
            <AppIcon name="minus" :size="16" />
          </button>
        </div>
        <datalist id="schema-field-type-options">
          <option v-for="type in typeSuggestions" :key="type" :value="type" />
        </datalist>
      </section>
      <p class="muted">
        Changes apply immediately. Data types use the connected database dialect; required fields
        and primary keys cannot contain NULL values.
      </p>
      <p v-if="error" class="mutation-warning" role="alert">{{ error }}</p>
      <div class="mutation-actions">
        <button type="button" class="button secondary" :disabled="saving" @click="dialog?.close()">
          Cancel
        </button>
        <button type="submit" class="button" :disabled="!canSave">
          {{ saving ? 'Applying…' : table ? 'Apply changes' : 'Create table' }}
        </button>
      </div>
    </form>
  </dialog>
</template>

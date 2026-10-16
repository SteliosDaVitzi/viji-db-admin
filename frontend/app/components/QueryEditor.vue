<script setup lang="ts">
const props = defineProps<{
  initialSql: string
  busy: boolean
  activeQuery: string | null
  demo: boolean
}>()
const emit = defineEmits<{ run: [sql: string]; activate: [id: string]; close: [id: string] }>()
interface SavedQuery {
  id: string
  name: string
  sql: string
}
interface QueryDraft {
  id: string
  number: number
  sql: string
  savedId: string | null
}
const storageKey = 'webdb.saved-queries.v1'
let nextNumber = 1
const drafts = ref<QueryDraft[]>([
  { id: 'query-1', number: 1, sql: props.initialSql, savedId: null },
])
const activeId = ref('query-1')
const activeDraft = computed(() => drafts.value.find((draft) => draft.id === activeId.value)!)
const sql = computed({
  get: () => activeDraft.value.sql,
  set: (value: string) => {
    activeDraft.value.sql = value
  },
})
const saved = ref<SavedQuery[]>([])
const selected = computed(() => saved.value.find((query) => query.id === activeDraft.value.savedId))
const lineCount = computed(() => sql.value.split('\n').length)
const scrollTop = ref(0)
const naming = ref(false)
const name = ref('')
const feedback = ref('')
const storageError = ref('')
const library = useTemplateRef<HTMLDetailsElement>('library')
const editor = useTemplateRef<HTMLTextAreaElement>('editor')
const nameInput = useTemplateRef<HTMLInputElement>('nameInput')
function closeLibrary(event: PointerEvent) {
  if (library.value && !library.value.contains(event.target as Node)) library.value.open = false
}
onMounted(() => {
  document.addEventListener('pointerdown', closeLibrary)
  try {
    const stored: unknown = JSON.parse(localStorage.getItem(storageKey) || '[]')
    if (!Array.isArray(stored) || stored.length > 100 || !stored.every(isSavedQuery)) {
      storageError.value = 'Saved queries could not be read. Browser data has not been changed.'
      return
    }
    saved.value = stored
  } catch {
    storageError.value = 'Browser storage is unavailable. Queries cannot be saved here.'
  }
})
onUnmounted(() => document.removeEventListener('pointerdown', closeLibrary))
function isSavedQuery(value: unknown): value is SavedQuery {
  if (!value || typeof value !== 'object') return false
  const query = value as Partial<SavedQuery>
  return (
    typeof query.id === 'string' &&
    typeof query.name === 'string' &&
    query.name.length > 0 &&
    query.name.length <= 80 &&
    typeof query.sql === 'string' &&
    query.sql.length <= 12_000
  )
}
watch(
  () => props.initialSql,
  (value, previous) => {
    const draft = activeDraft.value
    if (!draft.savedId && draft.sql === previous && !props.activeQuery) draft.sql = value
  },
)
watch(activeId, async () => {
  naming.value = false
  feedback.value = ''
  scrollTop.value = 0
  await nextTick()
  document
    .getElementById(`${activeId.value}-tab`)
    ?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  if (editor.value) {
    editor.value.scrollTop = 0
    editor.value.scrollLeft = 0
  }
})
function draftName(draft: QueryDraft) {
  const query = saved.value.find((item) => item.id === draft.savedId)
  return query ? `${query.name}${query.sql !== draft.sql ? ' •' : ''}` : `Query ${draft.number}`
}
function activate(id: string) {
  if (props.busy || id === activeId.value) return
  activeId.value = id
  emit('activate', id)
}
async function navigateTab(event: KeyboardEvent, index: number) {
  const last = drafts.value.length - 1
  const target =
    event.key === 'ArrowRight'
      ? (index + 1) % drafts.value.length
      : event.key === 'ArrowLeft'
        ? (index + last) % drafts.value.length
        : event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? last
            : null
  if (target === null || props.busy) return
  event.preventDefault()
  const draft = drafts.value[target]!
  activate(draft.id)
  await nextTick()
  document.getElementById(`${draft.id}-tab`)?.focus()
}
function closeDraft(id: string) {
  if (props.busy || drafts.value.length === 1) return
  const index = drafts.value.findIndex((draft) => draft.id === id)
  if (activeId.value === id) activate(drafts.value[index === 0 ? 1 : index - 1]!.id)
  drafts.value = drafts.value.filter((draft) => draft.id !== id)
  emit('close', id)
}
function persist(queries: SavedQuery[]) {
  try {
    localStorage.setItem(storageKey, JSON.stringify(queries))
    saved.value = queries
    storageError.value = ''
    return true
  } catch {
    storageError.value = 'Could not save changes. Check browser storage and try again.'
    return false
  }
}
async function startSave() {
  name.value = selected.value?.name || ''
  naming.value = true
  feedback.value = ''
  await nextTick()
  nameInput.value?.focus()
}
function save() {
  if (!name.value.trim() || !sql.value.trim() || sql.value.length > 12_000) return
  if (!selected.value && saved.value.length >= 100) {
    storageError.value = 'You have 100 saved queries. Remove one before saving another.'
    return
  }
  const query: SavedQuery = {
    id: selected.value?.id || crypto.randomUUID(),
    name: name.value.trim(),
    sql: sql.value,
  }
  const queries = selected.value
    ? saved.value.map((item) => (item.id === query.id ? query : item))
    : [...saved.value, query]
  if (persist(queries)) {
    activeDraft.value.savedId = query.id
    naming.value = false
    feedback.value = 'Saved in this browser'
  }
}
async function load(query: SavedQuery) {
  if (props.busy) return
  const existing = drafts.value.find((draft) => draft.savedId === query.id)
  if (existing) activate(existing.id)
  else await newQuery(query)
  if (library.value) library.value.open = false
  editor.value?.focus()
}
function remove(id: string) {
  if (persist(saved.value.filter((query) => query.id !== id))) {
    for (const draft of drafts.value) {
      if (draft.savedId === id) draft.savedId = null
    }
    feedback.value = 'Query removed'
  }
}
async function newQuery(query?: SavedQuery) {
  if (props.busy) return
  const number = ++nextNumber
  const draft: QueryDraft = {
    id: `query-${number}`,
    number,
    sql: query?.sql || props.initialSql,
    savedId: query?.id || null,
  }
  drafts.value.push(draft)
  activate(draft.id)
  await nextTick()
  editor.value?.focus()
}
function run() {
  if (!props.busy && sql.value.trim()) emit('run', sql.value)
}
</script>

<template>
  <section class="query-space" aria-label="Query workspace">
    <div class="query-toolbar">
      <div class="query-tabs" role="tablist" aria-label="Queries">
        <div
          v-for="(draft, index) in drafts"
          :key="draft.id"
          class="query-tab"
          :class="{ active: activeId === draft.id }"
          role="presentation"
        >
          <button
            :id="`${draft.id}-tab`"
            class="query-tab-label"
            role="tab"
            :aria-selected="activeId === draft.id"
            aria-controls="query-panel"
            :tabindex="activeId === draft.id ? 0 : -1"
            :disabled="busy"
            :title="draftName(draft)"
            @click="activate(draft.id)"
            @keydown="navigateTab($event, index)"
          >
            {{ draftName(draft) }}
          </button>
          <button
            v-if="drafts.length > 1"
            class="query-tab-close"
            type="button"
            :disabled="busy"
            :aria-label="`Close ${draftName(draft)}`"
            @click="closeDraft(draft.id)"
          >
            <AppIcon name="close" :size="12" />
          </button>
        </div>
        <button
          class="icon-button query-add"
          type="button"
          :disabled="busy"
          title="New query"
          aria-label="New query"
          @click="newQuery()"
        >
          <AppIcon name="plus" :size="14" />
        </button>
      </div>
      <div class="query-actions">
        <details
          ref="library"
          class="query-library"
          @keydown.esc="library && (library.open = false)"
        >
          <summary>
            Saved<span v-if="saved.length"> {{ saved.length }}</span
            ><AppIcon name="down" :size="12" />
          </summary>
          <div class="query-library-panel">
            <p class="query-library-note">Saved in this browser</p>
            <p v-if="!saved.length" class="query-library-empty">Save a query to reuse it here.</p>
            <ul v-else>
              <li v-for="query in saved" :key="query.id">
                <button
                  class="query-load"
                  type="button"
                  :title="query.sql"
                  :disabled="busy"
                  @click="load(query)"
                >
                  {{ query.name }}
                </button>
                <button
                  class="icon-button"
                  type="button"
                  :aria-label="`Delete ${query.name}`"
                  :title="`Delete ${query.name}`"
                  @click="remove(query.id)"
                >
                  <AppIcon name="close" :size="13" />
                </button>
              </li>
            </ul>
          </div>
        </details>
        <button class="text-button" type="button" :disabled="!sql.trim()" @click="startSave">
          {{ selected ? 'Update' : 'Save' }}
        </button>
        <button
          class="button primary small query-run"
          type="button"
          :disabled="busy || !sql.trim()"
          title="Run query (Ctrl / ⌘ + Enter)"
          @click="run"
        >
          <AppIcon name="play" :size="12" />{{ busy ? 'Running…' : 'Run' }}
        </button>
      </div>
    </div>
    <div id="query-panel" class="query-panel" role="tabpanel" :aria-labelledby="`${activeId}-tab`">
      <div class="query-code">
        <div class="query-gutter" aria-hidden="true">
          <div :style="{ transform: `translateY(-${scrollTop}px)` }">
            <span v-for="line in lineCount" :key="line">{{ line }}</span>
          </div>
        </div>
        <textarea
          id="sql-editor"
          ref="editor"
          v-model="sql"
          class="sql-editor"
          rows="5"
          wrap="off"
          aria-label="SQL query"
          maxlength="12000"
          spellcheck="false"
          autocapitalize="off"
          autocomplete="off"
          aria-describedby="query-help"
          placeholder="SELECT * FROM …"
          @input="feedback = ''"
          @scroll="scrollTop = ($event.target as HTMLTextAreaElement).scrollTop"
          @keydown="
            ($event.ctrlKey || $event.metaKey) &&
            $event.key === 'Enter' &&
            ($event.preventDefault(), run())
          "
        />
      </div>
    </div>
    <form
      v-if="naming"
      class="query-save-form"
      @submit.prevent="save"
      @keydown.esc="naming = false"
    >
      <input
        ref="nameInput"
        v-model="name"
        aria-label="Query name"
        placeholder="Name this query"
        maxlength="80"
        required
      />
      <button class="button secondary small" type="submit" :disabled="!name.trim()">
        {{ selected ? 'Update query' : 'Save query' }}
      </button>
      <button class="text-button" type="button" @click="naming = false">Cancel</button>
    </form>
    <div class="query-footnote">
      <span id="query-help">{{
        activeQuery && sql.trim() !== activeQuery
          ? 'Query changed · Run to update results'
          : demo
            ? 'Sample: SELECT * FROM a table, with optional LIMIT'
            : 'Read only · Ctrl / ⌘ + Enter to run'
      }}</span>
      <span v-if="feedback" role="status">{{ feedback }}</span>
    </div>
    <p v-if="storageError" class="query-storage-error" role="alert">{{ storageError }}</p>
  </section>
</template>

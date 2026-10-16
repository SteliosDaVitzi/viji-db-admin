<script setup lang="ts">
import type { DatabaseSettings } from '../../shared/database'
const props = defineProps<{ database: string; provider: string }>()
const emit = defineEmits<{ close: [] }>()
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const result = shallowRef<DatabaseSettings | null>(null)
const loading = ref(true)
const error = ref('')
const filter = ref('')
const category = ref('')
const controller = new AbortController()
const vendor = computed(() => (props.provider === 'postgres' ? 'PostgreSQL' : props.provider))
const categories = computed(() => [
  ...new Set(result.value?.settings.map((s) => String(s.category)) || []),
])
const settings = computed(
  () =>
    result.value?.settings.filter(
      (s) =>
        (!category.value || s.category === category.value) &&
        Object.values(s).some((value) =>
          String(value ?? '')
            .toLowerCase()
            .includes(filter.value.toLowerCase()),
        ),
    ) || [],
)
function display(value: unknown) {
  return value === null
    ? 'Not available'
    : typeof value === 'object'
      ? JSON.stringify(value)
      : String(value)
}
async function load() {
  loading.value = true
  error.value = ''
  try {
    result.value = await $fetch<DatabaseSettings>('/api/database-settings', {
      signal: controller.signal,
    })
  } catch (cause) {
    if (!controller.signal.aborted) {
      const failure = cause as { statusCode?: number; status?: number }
      const status = failure.statusCode || failure.status
      error.value =
        status === 404
          ? 'The running backend needs an update to support database settings. Restart the WebDB backend, then reconnect and try again.'
          : status === 401
            ? 'Your connection session expired or the backend restarted. Reconnect to load database settings.'
            : status === 403
              ? 'This database user does not have permission to read settings. Check the user’s database permissions, then try again.'
              : 'Could not load database settings. Check your connection and database permissions, then try again.'
    }
  } finally {
    loading.value = false
  }
}
onMounted(() => {
  dialog.value?.showModal()
  if (props.provider === 'demo') loading.value = false
  else void load()
})
onScopeDispose(() => controller.abort())
</script>
<template>
  <dialog
    ref="dialog"
    class="database-settings"
    aria-labelledby="settings-title"
    @close="emit('close')"
    @contextmenu.prevent
  >
    <div class="settings-heading">
      <div>
        <h2 id="settings-title">{{ database }} settings</h2>
        <p>{{ vendor }} · Read only</p>
      </div>
      <button class="icon-button" aria-label="Close database settings" @click="emit('close')">
        <AppIcon name="close" />
      </button>
    </div>
    <p v-if="provider === 'demo'">
      Sample data has no database server settings. Connect to a database to inspect its
      configuration.
    </p>
    <p v-else-if="loading" role="status">Loading database settings…</p>
    <div v-else-if="error" role="alert">
      {{ error }} <button class="text-button" @click="load">Try again</button>
    </div>
    <template v-else-if="result">
      <h3>Database properties</h3>
      <dl class="settings-properties">
        <template v-for="(value, name) in result.properties" :key="name"
          ><dt>{{ name }}</dt>
          <dd>{{ display(value) }}</dd></template
        >
      </dl>
      <h3>Effective server settings</h3>
      <p>
        Values visible to your database user, including database and role overrides. Sensitive
        values are redacted. WebDB uses a separate read-only connection; session settings describe
        that connection.
      </p>
      <div class="settings-filters">
        <label class="search-box"
          ><AppIcon name="search" :size="15" /><input
            v-model="filter"
            aria-label="Search database settings"
            placeholder="Search settings…"
        /></label>
        <select v-model="category" aria-label="Settings category">
          <option value="">All categories</option>
          <option v-for="name in categories" :key="name">{{ name }}</option>
        </select>
      </div>
      <p role="status">{{ settings.length }} of {{ result.settings.length }} settings</p>
      <details v-for="setting in settings" :key="String(setting.name)" class="setting-detail">
        <summary>
          <strong>{{ setting.name }}</strong
          ><span>{{ display(setting.setting) }} {{ setting.unit || '' }}</span>
        </summary>
        <p>{{ setting.short_desc }}</p>
        <dl class="settings-properties">
          <template v-for="(value, name) in setting" :key="name"
            ><dt>{{ name }}</dt>
            <dd>{{ display(value) }}</dd></template
          >
        </dl>
      </details>
      <p v-if="!settings.length">No settings match these filters.</p>
    </template>
  </dialog>
</template>

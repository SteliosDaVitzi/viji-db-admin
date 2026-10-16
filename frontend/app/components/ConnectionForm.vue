<script setup lang="ts">
import { providerOptions } from '../providers'
import type { ConnectionInput, Provider, SavedConnection } from '../../shared/database'
defineProps<{
  busy: boolean
  error: string
  providers: Provider[]
  savedConnections: SavedConnection[]
  savingEnabled: boolean
}>()
const emit = defineEmits<{
  connect: [input: ConnectionInput]
  reconnect: [id: string]
  remove: [id: string]
}>()
const save = ref(false)
const saveName = ref('')
const instanceHintId = useId()
const form = reactive<ConnectionInput>({
  provider: 'postgres',
  host: 'localhost',
  port: 5432,
  database: '',
  username: '',
  password: '',
  ssl: true,
  allDatabases: false,
})
const options = computed(() => providerOptions(form.provider))
watch(
  () => form.provider,
  () => {
    form.port = options.value.port
    form.password = ''
  },
)
function submit() {
  emit('connect', {
    ...form,
    database: form.allDatabases ? '' : form.database,
    saveName: save.value ? saveName.value.trim() : undefined,
  })
  form.password = ''
}
</script>
<template>
  <form class="connection-form" @submit.prevent="submit">
    <div class="connection-heading">
      <span class="connection-symbol"><AppIcon name="database" :size="26" /></span>
      <div>
        <h1>Connect to your database</h1>
        <p>Your data. A little more clarity.</p>
      </div>
    </div>
    <section
      v-if="savedConnections.length"
      class="saved-connections"
      aria-label="Saved connections"
    >
      <h2>Saved connections</h2>
      <div v-for="saved in savedConnections" :key="saved.id" class="saved-connection">
        <button
          type="button"
          class="text-button"
          :disabled="busy"
          @click="emit('reconnect', saved.id)"
        >
          <AppIcon name="database" :size="14" />{{ saved.name }}
        </button>
        <button
          type="button"
          class="icon-button"
          :disabled="busy"
          :aria-label="`Remove ${saved.name}`"
          title="Remove saved connection"
          @click="emit('remove', saved.id)"
        >
          <AppIcon name="close" :size="13" />
        </button>
      </div>
    </section>
    <label
      >Database type<select v-model="form.provider" :disabled="busy">
        <option
          v-for="provider in providers.filter((p) => p.id !== 'demo')"
          :key="provider.id"
          :value="provider.id"
        >
          {{ provider.name }}
        </option>
        <option v-if="!providers.length" value="postgres">PostgreSQL</option>
      </select></label
    >
    <div class="form-row host-row">
      <label
        >Host<input
          v-model.trim="form.host"
          required
          maxlength="253"
          autocomplete="off"
          placeholder="localhost"
          :disabled="busy" /></label
      ><label
        >Port<input
          v-model.number="form.port"
          type="number"
          min="1"
          max="65535"
          required
          :disabled="busy"
      /></label>
    </div>
    <label class="checkbox-label scope-toggle">
      <input
        v-model="form.allDatabases"
        type="checkbox"
        role="switch"
        :disabled="busy"
        :aria-describedby="instanceHintId"
      />
      <span>Browse all databases in this instance</span>
    </label>
    <p :id="instanceHintId" class="field-help">
      {{
        form.allDatabases
          ? `Discover databases your user can access, then choose one in the explorer. Connects ${options.maintenance}.`
          : 'Connect to one database by name.'
      }}
    </p>
    <label v-if="!form.allDatabases"
      >Database<input
        v-model.trim="form.database"
        required
        maxlength="128"
        autocomplete="off"
        placeholder="Database name"
        :disabled="busy"
    /></label>
    <div class="form-row">
      <label
        >Username<input
          v-model="form.username"
          required
          maxlength="128"
          autocomplete="username"
          :placeholder="options.username"
          :disabled="busy" /></label
      ><label
        >Password<input
          v-model="form.password"
          type="password"
          maxlength="4096"
          autocomplete="current-password"
          placeholder="Enter password"
          :disabled="busy"
      /></label>
    </div>
    <label class="checkbox-label"
      ><input v-model="form.ssl" type="checkbox" :disabled="busy" /><span
        >Verify TLS connection</span
      ><span
        class="hint"
        title="Verifies the server certificate and hostname. Turn off only for a trusted local database."
        tabindex="0"
        aria-label="TLS verifies the server certificate and hostname. Turn off only for a trusted local database."
        >?</span
      ></label
    >
    <label class="checkbox-label save-toggle"
      ><input v-model="save" type="checkbox" :disabled="busy || !savingEnabled" /><span
        >Save connection and credentials</span
      ></label
    >
    <p v-if="!savingEnabled" class="field-help">
      Saving is unavailable. Configure the backend connection vault to enable it.
    </p>
    <label v-if="save && savingEnabled"
      >Connection name<input
        v-model.trim="saveName"
        required
        maxlength="128"
        placeholder="e.g. Local development"
        :disabled="busy"
    /></label>
    <button class="button primary connect-button" type="submit" :disabled="busy">
      <span>{{
        busy ? 'Connecting…' : form.allDatabases ? 'Connect to instance' : 'Connect to database'
      }}</span
      ><AppIcon name="arrow" />
    </button>
  </form>
</template>

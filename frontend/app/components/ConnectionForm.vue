<script setup lang="ts">
import type { ConnectionInput, Provider } from '../../shared/database'
defineProps<{ busy: boolean; error: string; providers: Provider[] }>()
const emit = defineEmits<{ connect: [input: ConnectionInput]; demo: [] }>()
const form = reactive<ConnectionInput>({
  provider: 'postgres',
  host: 'localhost',
  port: 5432,
  database: '',
  username: '',
  password: '',
  ssl: true,
})
function submit() {
  emit('connect', { ...form })
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
    <label
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
          placeholder="postgres"
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
    <p v-if="error" class="error-message" role="alert">{{ error }}</p>
    <button class="button primary connect-button" type="submit" :disabled="busy">
      <span>{{ busy ? 'Connecting…' : 'Connect to database' }}</span
      ><AppIcon name="arrow" />
    </button>
    <div class="form-footnote">
      <AppIcon name="lock" :size="13" /><span>Credentials stay in memory for this session.</span>
    </div>
    <div class="sample-link">
      <span>Just looking around?</span
      ><button type="button" class="text-button" :disabled="busy" @click="emit('demo')">
        Open sample workspace <span aria-hidden="true">↗</span>
      </button>
    </div>
  </form>
</template>

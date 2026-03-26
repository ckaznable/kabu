<script setup lang="ts">
import { ref } from 'vue'
import { uploadPdf } from './api'

const uploading = ref(false)
const uploadResult = ref('')
const fileInput = ref<HTMLInputElement>()

async function handleUpload() {
  const file = fileInput.value?.files?.[0]
  if (!file) return
  uploading.value = true
  uploadResult.value = ''
  try {
    await uploadPdf(file)
    uploadResult.value = 'PDF uploaded, processing in background'
  } catch (e: unknown) {
    uploadResult.value = `Error: ${e instanceof Error ? e.message : e}`
  } finally {
    uploading.value = false
    if (fileInput.value) fileInput.value.value = ''
  }
}
</script>

<template>
  <div class="app">
    <nav class="navbar">
      <div class="nav-brand">Kabu</div>
      <div class="nav-links">
        <RouterLink to="/" class="nav-link">Overview</RouterLink>
        <RouterLink to="/settings" class="nav-link">Settings</RouterLink>
      </div>
      <div class="nav-upload">
        <input ref="fileInput" type="file" accept=".pdf" @change="handleUpload" hidden />
        <button class="btn btn-upload" @click="fileInput?.click()" :disabled="uploading">
          {{ uploading ? 'Processing...' : 'Upload PDF' }}
        </button>
        <span v-if="uploadResult" class="upload-result">{{ uploadResult }}</span>
      </div>
    </nav>
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
  background: var(--bg);
  color: var(--text);
}

.navbar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.75rem 1.5rem;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}

.nav-brand {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--accent);
}

.nav-links {
  display: flex;
  gap: 0.5rem;
}

.nav-link {
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  text-decoration: none;
  color: var(--text-muted);
  font-size: 0.9rem;
  transition: background 0.15s;
}

.nav-link:hover,
.nav-link.router-link-exact-active {
  background: var(--hover);
  color: var(--text);
}

.nav-upload {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.upload-result {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.main-content {
  max-width: 1100px;
  margin: 0 auto;
  padding: 1.5rem;
}
</style>

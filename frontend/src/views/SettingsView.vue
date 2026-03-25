<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { fetchStocks, createStock, updateStock, deleteStock, type Stock } from '../api'

const stocks = ref<Stock[]>([])
const loading = ref(true)

const newSymbol = ref('')
const newName = ref('')
const newQty = ref<number>(0)
const newCost = ref<number>(0)

const editingId = ref<number | null>(null)
const editQty = ref<number>(0)
const editCost = ref<number>(0)
const editName = ref('')

async function load() {
  loading.value = true
  try {
    stocks.value = await fetchStocks()
  } finally {
    loading.value = false
  }
}

onMounted(load)

async function handleAdd() {
  const symbol = newSymbol.value.trim().toUpperCase()
  if (!symbol) return
  await createStock({
    symbol,
    name: newName.value.trim() || undefined,
    quantity: newQty.value,
    cost_basis: newCost.value,
  })
  newSymbol.value = ''
  newName.value = ''
  newQty.value = 0
  newCost.value = 0
  await load()
}

function startEdit(s: Stock) {
  editingId.value = s.id
  editQty.value = s.quantity
  editCost.value = s.cost_basis
  editName.value = s.name || ''
}

async function saveEdit(id: number) {
  await updateStock(id, {
    name: editName.value.trim() || undefined,
    quantity: editQty.value,
    cost_basis: editCost.value,
  })
  editingId.value = null
  await load()
}

function cancelEdit() {
  editingId.value = null
}

async function handleDelete(id: number) {
  if (!confirm('Remove this stock?')) return
  await deleteStock(id)
  await load()
}

const fmt = (n: number) =>
  n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
</script>

<template>
  <div>
    <h1>Settings</h1>

    <div v-if="loading" class="status">Loading...</div>

    <template v-else>
      <section class="section">
        <h2>Add Stock</h2>
        <form class="add-form" @submit.prevent="handleAdd">
          <input v-model="newSymbol" placeholder="Symbol (e.g. AAPL)" required />
          <input v-model="newName" placeholder="Name (optional)" />
          <input v-model.number="newQty" type="number" step="any" placeholder="Quantity" />
          <input v-model.number="newCost" type="number" step="any" placeholder="Total Cost" />
          <button class="btn" type="submit">Add</button>
        </form>
      </section>

      <section class="section">
        <h2>Holdings</h2>
        <div v-if="stocks.length === 0" class="status">No stocks added yet.</div>
        <table v-else class="data-table">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Name</th>
              <th class="num">Quantity</th>
              <th class="num">Total Cost</th>
              <th class="num">Avg Cost</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in stocks" :key="s.id">
              <td class="symbol">{{ s.symbol }}</td>
              <template v-if="editingId === s.id">
                <td><input v-model="editName" class="edit-input" /></td>
                <td class="num"><input v-model.number="editQty" type="number" step="any" class="edit-input num-input" /></td>
                <td class="num"><input v-model.number="editCost" type="number" step="any" class="edit-input num-input" /></td>
                <td class="num">
                  ${{ editQty > 0 ? fmt(editCost / editQty) : '0.00' }}
                </td>
                <td class="actions">
                  <button class="btn btn-sm" @click="saveEdit(s.id)">Save</button>
                  <button class="btn btn-sm btn-ghost" @click="cancelEdit">Cancel</button>
                </td>
              </template>
              <template v-else>
                <td>{{ s.name || '-' }}</td>
                <td class="num">{{ s.quantity }}</td>
                <td class="num">${{ fmt(s.cost_basis) }}</td>
                <td class="num">
                  ${{ s.quantity > 0 ? fmt(s.cost_basis / s.quantity) : '0.00' }}
                </td>
                <td class="actions">
                  <button class="btn btn-sm" @click="startEdit(s)">Edit</button>
                  <button class="btn btn-sm btn-danger" @click="handleDelete(s.id)">Delete</button>
                </td>
              </template>
            </tr>
          </tbody>
        </table>
      </section>
    </template>
  </div>
</template>

<style scoped>
h1 {
  font-size: 1.4rem;
  font-weight: 600;
  margin-bottom: 1.25rem;
}

h2 {
  font-size: 1.1rem;
  font-weight: 500;
  margin-bottom: 0.75rem;
}

.section {
  margin-bottom: 2rem;
}

.status {
  color: var(--text-muted);
  padding: 1rem 0;
}

.add-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
}

.add-form input {
  padding: 0.5rem 0.75rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  font-size: 0.9rem;
  width: 160px;
}

.add-form input:focus {
  outline: none;
  border-color: var(--accent);
}

.symbol {
  font-weight: 600;
  color: var(--accent);
}

.edit-input {
  padding: 0.3rem 0.5rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text);
  font-size: 0.85rem;
  width: 100%;
}

.num-input {
  text-align: right;
}

.actions {
  display: flex;
  gap: 0.25rem;
}
</style>

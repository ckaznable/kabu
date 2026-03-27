<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { fetchStocks, fetchExchangeRates, createStock, updateStock, deleteStock, type Stock, type ExchangeRate } from '../api'

const stocks = ref<Stock[]>([])
const rates = ref<ExchangeRate[]>([])
const loading = ref(true)
const displayCurrency = ref(localStorage.getItem('kabu-display-currency') || 'USD')

const newSymbol = ref('')
const newName = ref('')
const newQty = ref<number>(0)
const newCost = ref<number>(0)
const newAvgCost = ref<number>(0)
const newType = ref('stock')

const editingId = ref<number | null>(null)
const editQty = ref<number>(0)
const editCost = ref<number>(0)
const editAvgCost = ref<number>(0)
const editName = ref('')

async function load() {
  loading.value = true
  try {
    const [s, r] = await Promise.all([fetchStocks(), fetchExchangeRates()])
    stocks.value = s
    rates.value = r
  } finally {
    loading.value = false
  }
}

onMounted(load)

// --- Add form sync ---
function onNewTotalInput() {
  if (newQty.value > 0) newAvgCost.value = +(newCost.value / newQty.value).toFixed(4)
}
function onNewAvgInput() {
  newCost.value = +(newAvgCost.value * newQty.value).toFixed(4)
}
function onNewQtyInput() {
  if (newAvgCost.value > 0) newCost.value = +(newAvgCost.value * newQty.value).toFixed(4)
  else if (newQty.value > 0) newAvgCost.value = +(newCost.value / newQty.value).toFixed(4)
}

// --- Edit form sync ---
function onEditTotalInput() {
  if (editQty.value > 0) editAvgCost.value = +(editCost.value / editQty.value).toFixed(4)
}
function onEditAvgInput() {
  editCost.value = +(editAvgCost.value * editQty.value).toFixed(4)
}
function onEditQtyInput() {
  if (editAvgCost.value > 0) editCost.value = +(editAvgCost.value * editQty.value).toFixed(4)
  else if (editQty.value > 0) editAvgCost.value = +(editCost.value / editQty.value).toFixed(4)
}

async function handleAdd() {
  const symbol = newSymbol.value.trim().toUpperCase()
  if (!symbol) return
  await createStock({
    symbol,
    name: newName.value.trim() || undefined,
    quantity: newQty.value,
    cost_basis: newCost.value,
    asset_type: newType.value,
  })
  newSymbol.value = ''
  newName.value = ''
  newQty.value = 0
  newCost.value = 0
  newAvgCost.value = 0
  newType.value = 'stock'
  await load()
}

function startEdit(s: Stock) {
  editingId.value = s.id
  editQty.value = s.quantity
  editCost.value = s.cost_basis
  editAvgCost.value = s.quantity > 0 ? +(s.cost_basis / s.quantity).toFixed(4) : 0
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

function onCurrencyChange() {
  localStorage.setItem('kabu-display-currency', displayCurrency.value)
}

const availableCurrencies = computed(() =>
  ['USD', ...rates.value.map(r => r.currency)]
)

const fmt = (n: number) =>
  n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
</script>

<template>
  <div>
    <h1>Settings</h1>

    <div v-if="loading" class="status">Loading...</div>

    <template v-else>
      <section class="section">
        <h2>Display Currency</h2>
        <p class="hint">Portfolio values on the Overview page will be shown in this currency.</p>
        <div class="currency-row">
          <select v-model="displayCurrency" class="select-input" @change="onCurrencyChange">
            <option v-for="c in availableCurrencies" :key="c" :value="c">{{ c }}</option>
          </select>
          <span class="currency-note" v-if="displayCurrency !== 'USD'">
            Converted from USD using ECB exchange rates
          </span>
        </div>
      </section>

      <section class="section">
        <h2>Add Stock</h2>
        <form class="add-form" @submit.prevent="handleAdd">
          <label class="form-field">
            <span class="form-label">Type</span>
            <select v-model="newType" class="select-input">
              <option value="stock">Stock</option>
              <option value="crypto">Crypto</option>
            </select>
          </label>
          <label class="form-field">
            <span class="form-label">Symbol</span>
            <input v-model="newSymbol" :placeholder="newType === 'crypto' ? 'e.g. BTC' : 'e.g. AAPL'" required />
          </label>
          <label class="form-field">
            <span class="form-label">Name</span>
            <input v-model="newName" placeholder="optional" />
          </label>
          <label class="form-field">
            <span class="form-label">Quantity</span>
            <input v-model.number="newQty" type="number" step="any" @input="onNewQtyInput" />
          </label>
          <label class="form-field">
            <span class="form-label">Total Cost</span>
            <input v-model.number="newCost" type="number" step="any" @input="onNewTotalInput" />
          </label>
          <label class="form-field">
            <span class="form-label">Avg Cost</span>
            <input v-model.number="newAvgCost" type="number" step="any" @input="onNewAvgInput" />
          </label>
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
              <th>Type</th>
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
              <td><span class="type-badge" :class="s.asset_type">{{ s.asset_type }}</span></td>
              <template v-if="editingId === s.id">
                <td><input v-model="editName" class="edit-input" /></td>
                <td class="num"><input v-model.number="editQty" type="number" step="any" class="edit-input num-input" @input="onEditQtyInput" /></td>
                <td class="num"><input v-model.number="editCost" type="number" step="any" class="edit-input num-input" @input="onEditTotalInput" /></td>
                <td class="num"><input v-model.number="editAvgCost" type="number" step="any" class="edit-input num-input" @input="onEditAvgInput" /></td>
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

.hint {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-bottom: 0.5rem;
}

.currency-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.currency-note {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.add-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: flex-end;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.form-label {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.add-form input,
.select-input {
  padding: 0.5rem 0.75rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  font-size: 0.9rem;
  width: 140px;
}

.select-input {
  width: 100px;
  cursor: pointer;
}

.add-form input:focus,
.select-input:focus {
  outline: none;
  border-color: var(--accent);
}

.symbol {
  font-weight: 600;
  color: var(--accent);
}

.type-badge {
  font-size: 0.7rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  text-transform: uppercase;
  font-weight: 500;
}

.type-badge.stock {
  background: #1e3a5f;
  color: #6c8aff;
}

.type-badge.crypto {
  background: #1a3d2e;
  color: #34d399;
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

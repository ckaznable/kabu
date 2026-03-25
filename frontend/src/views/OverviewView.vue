<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { fetchPortfolio, type PortfolioSummary } from '../api'

const portfolio = ref<PortfolioSummary | null>(null)
const loading = ref(true)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    portfolio.value = await fetchPortfolio()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load'
  } finally {
    loading.value = false
  }
}

onMounted(load)

const fmt = (n: number) => n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
const fmtPct = (n: number) => n.toFixed(2)
const cls = (n: number) => (n > 0 ? 'positive' : n < 0 ? 'negative' : '')

const hasData = computed(() => portfolio.value && portfolio.value.holdings.length > 0)
</script>

<template>
  <div>
    <div class="header-row">
      <h1>Portfolio Overview</h1>
      <button class="btn btn-sm" @click="load" :disabled="loading">Refresh</button>
    </div>

    <div v-if="loading" class="status">Loading...</div>
    <div v-else-if="error" class="status error">{{ error }}</div>
    <div v-else-if="!hasData" class="status">
      No stocks yet. Go to <RouterLink to="/settings">Settings</RouterLink> to add stocks.
    </div>

    <template v-if="portfolio && hasData">
      <div class="summary-cards">
        <div class="card">
          <div class="card-label">Total Value</div>
          <div class="card-value">${{ fmt(portfolio.total_value) }}</div>
        </div>
        <div class="card">
          <div class="card-label">Total Cost</div>
          <div class="card-value">${{ fmt(portfolio.total_cost) }}</div>
        </div>
        <div class="card">
          <div class="card-label">Gain / Loss</div>
          <div class="card-value" :class="cls(portfolio.total_gain_loss)">
            ${{ fmt(portfolio.total_gain_loss) }}
            <span class="pct">({{ fmtPct(portfolio.total_gain_loss_percent) }}%)</span>
          </div>
        </div>
      </div>

      <table class="data-table">
        <thead>
          <tr>
            <th>Symbol</th>
            <th>Name</th>
            <th class="num">Qty</th>
            <th class="num">Avg Cost</th>
            <th class="num">Price</th>
            <th class="num">Value</th>
            <th class="num">Gain/Loss</th>
            <th class="num">%</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="h in portfolio.holdings" :key="h.stock.id">
            <td class="symbol">{{ h.stock.symbol }}</td>
            <td>{{ h.stock.name || '-' }}</td>
            <td class="num">{{ h.stock.quantity }}</td>
            <td class="num">
              ${{ h.stock.quantity > 0 ? fmt(h.stock.cost_basis / h.stock.quantity) : '0.00' }}
            </td>
            <td class="num">{{ h.latest_price != null ? '$' + fmt(h.latest_price) : '-' }}</td>
            <td class="num">${{ fmt(h.current_value) }}</td>
            <td class="num" :class="cls(h.gain_loss)">${{ fmt(h.gain_loss) }}</td>
            <td class="num" :class="cls(h.gain_loss_percent)">{{ fmtPct(h.gain_loss_percent) }}%</td>
          </tr>
        </tbody>
      </table>
    </template>
  </div>
</template>

<style scoped>
.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.25rem;
}

h1 {
  font-size: 1.4rem;
  font-weight: 600;
}

.status {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.status.error {
  color: var(--red);
}

.summary-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem 1.25rem;
}

.card-label {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-bottom: 0.25rem;
}

.card-value {
  font-size: 1.5rem;
  font-weight: 600;
}

.pct {
  font-size: 0.9rem;
  font-weight: 400;
}

.symbol {
  font-weight: 600;
  color: var(--accent);
}

.positive { color: var(--green); }
.negative { color: var(--red); }
</style>

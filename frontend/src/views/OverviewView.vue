<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  fetchPortfolio,
  fetchPortfolioSnapshots,
  fetchExchangeRates,
  fetchTransactions,
  type PortfolioSummary,
  type ExchangeRate,
  type PortfolioSnapshot,
  type Transaction,
} from '../api'
import AllocationChart from '../components/AllocationChart.vue'
import GainLossChart from '../components/GainLossChart.vue'
import PerformanceChart from '../components/PerformanceChart.vue'
import AssetTypeChart from '../components/AssetTypeChart.vue'
import PortfolioTrendChart from '../components/PortfolioTrendChart.vue'

const portfolio = ref<PortfolioSummary | null>(null)
const snapshots = ref<PortfolioSnapshot[]>([])
const rates = ref<ExchangeRate[]>([])
const transactions = ref<Transaction[]>([])
const loading = ref(true)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [p, s, r, t] = await Promise.all([
      fetchPortfolio(),
      fetchPortfolioSnapshots(),
      fetchExchangeRates(),
      fetchTransactions(),
    ])
    portfolio.value = p
    snapshots.value = s
    rates.value = r
    transactions.value = t
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load'
  } finally {
    loading.value = false
  }
}

onMounted(load)

const rateMap = computed(() => {
  const map: Record<string, number> = {}
  for (const r of rates.value) map[r.currency] = r.rate
  return map
})

const displayCurrency = ref(localStorage.getItem('kabu-display-currency') || 'USD')
const displayRate = computed(() => {
  if (displayCurrency.value === 'USD') return 1
  return rateMap.value[displayCurrency.value] ?? 1
})
const displaySymbol = computed(() => displayCurrency.value === 'USD' ? '$' : displayCurrency.value + ' ')

const fmtDisplay = (usd: number) => fmt(usd * displayRate.value)

const fmt = (n: number) => n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
const fmtPct = (n: number) => n.toFixed(2)
const portfolioWeight = (currentValue: number) => {
  const totalValue = portfolio.value?.total_value ?? 0
  return totalValue > 0 ? (currentValue / totalValue) * 100 : 0
}
const cls = (n: number) => (n > 0 ? 'positive' : n < 0 ? 'negative' : '')
const fmtDateTime = (s: string | null) => {
  if (!s) return '-'
  const d = new Date(s.replace(' ', 'T') + 'Z')
  return Number.isNaN(d.getTime()) ? s : d.toLocaleString('en-US')
}

const hasData = computed(() => portfolio.value && portfolio.value.holdings.length > 0)
const recentTransactions = computed(() => transactions.value.slice(0, 20))

type GainLossFilter = 'all' | 'stock' | 'crypto'
const gainLossFilter = ref<GainLossFilter>('all')
type HoldingsSortKey = 'gain_loss' | 'gain_loss_percent'
type SortDirection = 'asc' | 'desc'
const holdingsSortKey = ref<HoldingsSortKey | null>(null)
const holdingsSortDirection = ref<SortDirection>('desc')

const filteredGainLossHoldings = computed(() => {
  const holdings = portfolio.value?.holdings ?? []
  if (gainLossFilter.value === 'all') return holdings
  return holdings.filter((h) => h.stock.asset_type.toLowerCase() === gainLossFilter.value)
})

const gainLossSummary = computed(() => {
  const totalCost = filteredGainLossHoldings.value.reduce((sum, h) => sum + h.stock.cost_basis, 0)
  const totalValue = filteredGainLossHoldings.value.reduce((sum, h) => sum + h.current_value, 0)
  const totalGainLoss = totalValue - totalCost
  const totalGainLossPercent = totalCost > 0 ? (totalGainLoss / totalCost) * 100 : 0
  return { totalGainLoss, totalGainLossPercent }
})

const gainLossFilterLabel = computed(() => {
  if (gainLossFilter.value === 'stock') return 'Stock'
  if (gainLossFilter.value === 'crypto') return 'Crypto'
  return 'Stock + Crypto'
})

const dividendTransactions = computed(() =>
  transactions.value.filter(tx => tx.transaction_type === 'DIVIDEND')
)
const totalDividends = computed(() =>
  dividendTransactions.value.reduce((sum, tx) => sum + tx.total_amount, 0)
)
const recentDividends = computed(() => dividendTransactions.value.slice(0, 5))

const sortedHoldings = computed(() => {
  const holdings = portfolio.value?.holdings ?? []
  if (!holdingsSortKey.value) return holdings

  const direction = holdingsSortDirection.value === 'asc' ? 1 : -1
  return [...holdings].sort((a, b) => (a[holdingsSortKey.value!] - b[holdingsSortKey.value!]) * direction)
})

function toggleHoldingsSort(key: HoldingsSortKey) {
  if (holdingsSortKey.value === key) {
    holdingsSortDirection.value = holdingsSortDirection.value === 'desc' ? 'asc' : 'desc'
    return
  }

  holdingsSortKey.value = key
  holdingsSortDirection.value = 'desc'
}

function sortIndicator(key: HoldingsSortKey) {
  if (holdingsSortKey.value !== key) return ''
  return holdingsSortDirection.value === 'desc' ? '▼' : '▲'
}

const latestTradingDate = computed(() => {
  const timestamps = (portfolio.value?.holdings ?? [])
    .map((h) => h.latest_price_timestamp?.slice(0, 10) ?? null)
    .filter((value): value is string => value !== null)

  if (timestamps.length === 0) return null
  return timestamps.sort((a, b) => b.localeCompare(a))[0]
})

const topMovers = computed(() => {
  if (!latestTradingDate.value) return []

  return (portfolio.value?.holdings ?? [])
    .filter((h) =>
      h.latest_price_timestamp?.startsWith(latestTradingDate.value!) &&
      h.latest_change_percent != null
    )
    .sort((a, b) => Math.abs(b.latest_change_percent ?? 0) - Math.abs(a.latest_change_percent ?? 0))
    .slice(0, 10)
})
</script>

<template>
  <div>
    <div class="header-row">
      <h1>Portfolio Overview</h1>
      <button class="btn btn-sm" @click="load" :disabled="loading">Refresh</button>
    </div>

    <div v-if="loading" class="status">Loading...</div>
    <div v-else-if="error" class="status error">{{ error }}</div>
    <div v-else-if="!hasData && dividendTransactions.length === 0" class="status">
      No stocks yet. Go to <RouterLink to="/settings">Settings</RouterLink> to add stocks.
    </div>

    <template v-if="portfolio">
      <template v-if="hasData || dividendTransactions.length > 0">
        <div class="summary-cards">
          <div class="card">
            <div class="card-label">Total Value</div>
            <div class="card-value">{{ displaySymbol }}{{ fmtDisplay(portfolio.total_value) }}</div>
          </div>
          <div class="card">
            <div class="card-label">Total Cost</div>
            <div class="card-value">{{ displaySymbol }}{{ fmtDisplay(portfolio.total_cost) }}</div>
          </div>
          <div class="card">
            <div class="card-label">Gain / Loss</div>
            <div class="card-value" :class="cls(gainLossSummary.totalGainLoss)">
              {{ displaySymbol }}{{ fmtDisplay(gainLossSummary.totalGainLoss) }}
              <span class="pct">({{ fmtPct(gainLossSummary.totalGainLossPercent) }}%)</span>
            </div>
            <div class="card-sub">View: {{ gainLossFilterLabel }}</div>
          </div>
          <div class="card">
            <div class="card-label">Dividend Income</div>
            <div class="card-value positive">{{ displaySymbol }}{{ fmtDisplay(totalDividends) }}</div>
          </div>
        </div>

        <template v-if="hasData">
          <div v-if="displayCurrency !== 'USD' && displayRate > 1" class="rate-note">
            1 USD = {{ displayRate.toFixed(2) }} {{ displayCurrency }}
          </div>

          <div class="charts-row">
            <div class="chart-card chart-span-2">
              <h3>Portfolio Value Trend</h3>
              <PortfolioTrendChart :snapshots="snapshots" />
            </div>
            <div class="chart-card">
              <h3>Allocation</h3>
              <AllocationChart :holdings="portfolio.holdings" />
            </div>
            <div class="chart-card">
              <div class="chart-head">
                <h3>Gain / Loss ($)</h3>
                <div class="toggle-group">
                  <button class="toggle-btn" :class="{ active: gainLossFilter === 'all' }" @click="gainLossFilter = 'all'">Stock + Crypto</button>
                  <button class="toggle-btn" :class="{ active: gainLossFilter === 'stock' }" @click="gainLossFilter = 'stock'">Stock</button>
                  <button class="toggle-btn" :class="{ active: gainLossFilter === 'crypto' }" @click="gainLossFilter = 'crypto'">Crypto</button>
                </div>
              </div>
              <GainLossChart :holdings="filteredGainLossHoldings" />
            </div>
            <div class="chart-card">
              <h3>Return (%)</h3>
              <PerformanceChart :holdings="portfolio.holdings" />
            </div>
            <div class="chart-card">
              <h3>Stock / Crypto</h3>
              <AssetTypeChart :holdings="portfolio.holdings" />
            </div>
          </div>

          <div v-if="recentDividends.length > 0" class="dividend-card">
            <div class="section-head">
              <h3>Recent Dividends</h3>
              <RouterLink to="/settings" class="section-link">Manage Transactions</RouterLink>
            </div>
            <table class="data-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Symbol</th>
                  <th class="num">Amount</th>
                  <th class="num">Price</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="tx in recentDividends" :key="tx.id">
                  <td>{{ tx.transaction_date || '-' }}</td>
                  <td class="symbol">{{ tx.symbol }}</td>
                  <td class="num positive">{{ displaySymbol }}{{ fmtDisplay(tx.total_amount) }}</td>
                  <td class="num">{{ tx.price > 0 ? displaySymbol + fmtDisplay(tx.price) : '-' }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-if="topMovers.length > 0" class="dividend-card">
            <div class="section-head">
              <div>
                <h3>Top 10 Movers</h3>
                <div class="section-meta">Latest trading day: {{ latestTradingDate }}</div>
              </div>
            </div>
            <table class="data-table">
              <thead>
                <tr>
                  <th>Symbol</th>
                  <th>Name</th>
                  <th>Type</th>
                  <th class="num">Price</th>
                  <th class="num">Change</th>
                  <th class="num">Change %</th>
                  <th>Time</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="h in topMovers" :key="`mover-${h.stock.id}`">
                  <td class="symbol">{{ h.stock.symbol }}</td>
                  <td>{{ h.stock.name || '-' }}</td>
                  <td><span class="type-badge" :class="h.stock.asset_type">{{ h.stock.asset_type }}</span></td>
                  <td class="num">{{ h.latest_price != null ? displaySymbol + fmtDisplay(h.latest_price) : '-' }}</td>
                  <td class="num" :class="cls(h.latest_change ?? 0)">
                    {{ h.latest_change != null ? displaySymbol + fmtDisplay(h.latest_change) : '-' }}
                  </td>
                  <td class="num" :class="cls(h.latest_change_percent ?? 0)">
                    {{ h.latest_change_percent != null ? fmtPct(h.latest_change_percent) + '%' : '-' }}
                  </td>
                  <td>{{ fmtDateTime(h.latest_price_timestamp ?? null) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <table class="data-table">
            <thead>
              <tr>
                <th>Symbol</th>
                <th>Name</th>
                <th>Type</th>
                <th class="num">Qty</th>
                <th class="num">Avg Cost</th>
                <th class="num">Price</th>
                <th class="num">Value</th>
                <th class="num">Portfolio %</th>
                <th class="num">
                  <button class="sort-btn" @click="toggleHoldingsSort('gain_loss')">
                    Gain/Loss <span class="sort-indicator">{{ sortIndicator('gain_loss') }}</span>
                  </button>
                </th>
                <th class="num">
                  <button class="sort-btn" @click="toggleHoldingsSort('gain_loss_percent')">
                    % <span class="sort-indicator">{{ sortIndicator('gain_loss_percent') }}</span>
                  </button>
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="h in sortedHoldings" :key="h.stock.id">
                <td class="symbol">{{ h.stock.symbol }}</td>
                <td>{{ h.stock.name || '-' }}</td>
                <td><span class="type-badge" :class="h.stock.asset_type">{{ h.stock.asset_type }}</span></td>
                <td class="num">{{ h.stock.quantity }}</td>
                <td class="num">
                  {{ displaySymbol }}{{ h.stock.quantity > 0 ? fmtDisplay(h.stock.cost_basis / h.stock.quantity) : '0.00' }}
                </td>
                <td class="num">{{ h.latest_price != null ? displaySymbol + fmtDisplay(h.latest_price) : '-' }}</td>
                <td class="num">{{ displaySymbol }}{{ fmtDisplay(h.current_value) }}</td>
                <td class="num allocation-cell">
                  <div class="allocation-cell-inner">
                    <span>{{ fmtPct(portfolioWeight(h.current_value)) }}%</span>
                    <span class="allocation-track" aria-hidden="true">
                      <span
                        class="allocation-fill"
                        :style="{ width: `${Math.min(portfolioWeight(h.current_value), 100)}%` }"
                      ></span>
                    </span>
                  </div>
                </td>
                <td class="num" :class="cls(h.gain_loss)">{{ displaySymbol }}{{ fmtDisplay(h.gain_loss) }}</td>
                <td class="num" :class="cls(h.gain_loss_percent)">{{ fmtPct(h.gain_loss_percent) }}%</td>
              </tr>
            </tbody>
          </table>
        </template>
      </template>

      <div v-else class="status">
        No stocks yet. Go to <RouterLink to="/settings">Settings</RouterLink> to add stocks.
      </div>

      <div class="history-card">
        <h3>Transaction History</h3>
        <div v-if="recentTransactions.length === 0" class="status">
          No transaction history yet.
        </div>
        <table v-else class="data-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Symbol</th>
              <th>Type</th>
              <th class="num">Qty</th>
              <th class="num">Price</th>
              <th class="num">Total</th>
              <th>Source</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="tx in recentTransactions" :key="tx.id">
              <td>{{ fmtDateTime(tx.transaction_date || tx.created_at) }}</td>
              <td class="symbol">{{ tx.symbol }}</td>
              <td>{{ tx.transaction_type }}</td>
              <td class="num">{{ tx.quantity }}</td>
              <td class="num">{{ displaySymbol }}{{ fmtDisplay(tx.price) }}</td>
              <td class="num" :class="tx.transaction_type === 'DIVIDEND' ? 'positive' : ''">
                {{ displaySymbol }}{{ fmtDisplay(tx.total_amount) }}
              </td>
              <td>{{ tx.source || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
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

.charts-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.chart-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
}

.chart-span-2 {
  grid-column: span 2;
}

@media (max-width: 980px) {
  .chart-span-2 {
    grid-column: span 1;
  }
}

.chart-card h3 {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-muted);
  margin-bottom: 0.75rem;
}

.dividend-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
}

.section-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.section-head h3 {
  font-size: 0.95rem;
  font-weight: 600;
}

.section-link {
  color: var(--accent);
  text-decoration: none;
  font-size: 0.85rem;
}

.section-meta {
  color: var(--text-muted);
  font-size: 0.8rem;
  margin-top: 0.2rem;
}

.chart-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.chart-head h3 {
  margin-bottom: 0;
}

.toggle-group {
  display: inline-flex;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg);
}

.toggle-btn {
  border: none;
  border-right: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  padding: 0.3rem 0.55rem;
  font-size: 0.75rem;
  cursor: pointer;
}

.toggle-btn:last-child {
  border-right: none;
}

.toggle-btn.active {
  background: var(--accent);
  color: #fff;
}

@media (max-width: 720px) {
  .chart-head {
    flex-direction: column;
    align-items: flex-start;
  }
}

.history-card {
  margin-top: 1rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
}

.history-card h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
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

.converted-cards {
  margin-bottom: 1.5rem;
}

.card-sub {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
}

.positive { color: var(--green); }
.negative { color: var(--red); }

.sort-btn {
  border: none;
  background: transparent;
  color: inherit;
  font: inherit;
  cursor: pointer;
  padding: 0;
}

.sort-indicator {
  display: inline-block;
  min-width: 1ch;
  margin-left: 0.2rem;
}

.allocation-cell {
  min-width: 140px;
}

.allocation-cell-inner {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.65rem;
}

.allocation-track {
  width: 72px;
  height: 0.45rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  overflow: hidden;
}

.allocation-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--accent), color-mix(in srgb, var(--accent) 65%, white));
}
</style>

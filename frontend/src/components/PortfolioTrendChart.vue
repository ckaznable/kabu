<script setup lang="ts">
import { computed, ref } from 'vue'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LineElement,
  LinearScale,
  PointElement,
  Tooltip,
  Filler,
} from 'chart.js'
import type { PortfolioSnapshot } from '../api'

ChartJS.register(CategoryScale, LinearScale, LineElement, PointElement, Tooltip, Filler)

type RangeKey = '5d' | '1m' | '3m' | '6m' | '1y' | '5y'

const props = defineProps<{ snapshots: PortfolioSnapshot[] }>()
const selectedRange = ref<RangeKey>('1m')

const rangeDefs: Record<RangeKey, { label: string; days: number }> = {
  '5d': { label: '5D', days: 5 },
  '1m': { label: '1M', days: 30 },
  '3m': { label: '3M', days: 90 },
  '6m': { label: '6M', days: 180 },
  '1y': { label: '1Y', days: 365 },
  '5y': { label: '5Y', days: 365 * 5 },
}

const filteredSnapshots = computed(() => {
  if (props.snapshots.length === 0) return []
  const sorted = [...props.snapshots].sort((a, b) => a.timestamp.localeCompare(b.timestamp))
  const last = sorted[sorted.length - 1]
  const lastTime = new Date(last.timestamp.replace(' ', 'T') + 'Z').getTime()
  if (Number.isNaN(lastTime)) return sorted

  const days = rangeDefs[selectedRange.value].days
  const cutoff = lastTime - days * 24 * 60 * 60 * 1000
  return sorted.filter((s) => {
    const t = new Date(s.timestamp.replace(' ', 'T') + 'Z').getTime()
    return !Number.isNaN(t) && t >= cutoff
  })
})

const chartData = computed(() => ({
  labels: filteredSnapshots.value.map((s) =>
    new Date(s.timestamp.replace(' ', 'T') + 'Z').toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    })
  ),
  datasets: [
    {
      label: 'Total Value',
      data: filteredSnapshots.value.map((s) => s.total_value),
      borderColor: '#34d399',
      backgroundColor: 'rgba(52, 211, 153, 0.18)',
      borderWidth: 2,
      pointRadius: 2,
      pointHoverRadius: 4,
      fill: true,
      tension: 0.25,
    },
  ],
}))

const options = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (ctx: { parsed: { y: number | null } }) => {
          const value = ctx.parsed.y ?? 0
          return `$${value.toLocaleString('en-US', { maximumFractionDigits: 2 })}`
        },
      },
    },
  },
  scales: {
    x: {
      grid: { display: false },
      ticks: { color: '#e4e4e7' },
    },
    y: {
      grid: { color: '#2a2d3a' },
      ticks: {
        color: '#8b8d98',
        callback: (value: number | string) => `$${value}`,
      },
    },
  },
}
</script>

<template>
  <div>
    <div class="range-switch">
      <button
        v-for="(def, key) in rangeDefs"
        :key="key"
        class="range-btn"
        :class="{ active: selectedRange === key }"
        @click="selectedRange = key"
      >
        {{ def.label }}
      </button>
    </div>

    <div v-if="filteredSnapshots.length === 0" class="empty">
      No daily snapshots yet.
    </div>
    <div v-else class="chart-wrap">
      <Line :data="chartData" :options="options" />
    </div>
  </div>
</template>

<style scoped>
.range-switch {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin-bottom: 0.75rem;
}

.range-btn {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  border-radius: 6px;
  padding: 0.2rem 0.55rem;
  font-size: 0.75rem;
  cursor: pointer;
}

.range-btn.active {
  color: #111827;
  background: #34d399;
  border-color: #34d399;
}

.chart-wrap {
  height: 260px;
}

.empty {
  color: var(--text-muted);
  text-align: center;
  padding: 2rem 0;
}
</style>

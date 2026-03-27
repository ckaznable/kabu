<script setup lang="ts">
import { computed } from 'vue'
import { Doughnut } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js'
import type { HoldingSummary } from '../api'

ChartJS.register(ArcElement, Tooltip, Legend)

const props = defineProps<{ holdings: HoldingSummary[] }>()

const TYPE_COLORS: Record<string, string> = {
  stock: '#6c8aff',
  crypto: '#34d399',
}

const grouped = computed(() => {
  const map = new Map<string, number>()
  for (const h of props.holdings) {
    const type = h.stock.asset_type
    map.set(type, (map.get(type) ?? 0) + h.current_value)
  }
  return [...map.entries()].sort((a, b) => b[1] - a[1])
})

const chartData = computed(() => ({
  labels: grouped.value.map(([type]) => type.charAt(0).toUpperCase() + type.slice(1)),
  datasets: [
    {
      data: grouped.value.map(([, value]) => value),
      backgroundColor: grouped.value.map(([type]) => TYPE_COLORS[type] ?? '#8b8d98'),
      borderWidth: 0,
    },
  ],
}))

const options = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'right' as const,
      labels: { color: '#e4e4e7', padding: 12, font: { size: 12 } },
    },
    tooltip: {
      callbacks: {
        label: (ctx: { label: string; parsed: number }) =>
          `${ctx.label}: $${ctx.parsed.toLocaleString('en-US', { minimumFractionDigits: 2 })}`,
      },
    },
  },
}
</script>

<template>
  <div class="chart-wrap">
    <Doughnut :data="chartData" :options="options" />
  </div>
</template>

<style scoped>
.chart-wrap {
  height: 240px;
}
</style>

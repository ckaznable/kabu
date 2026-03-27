<script setup lang="ts">
import { computed } from 'vue'
import { Doughnut } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js'
import type { HoldingSummary } from '../api'

ChartJS.register(ArcElement, Tooltip, Legend)

const props = defineProps<{ holdings: HoldingSummary[] }>()

const COLORS = [
  '#6c8aff', '#34d399', '#f87171', '#fbbf24', '#a78bfa',
  '#f472b6', '#38bdf8', '#fb923c', '#4ade80', '#e879f9',
]

const chartData = computed(() => ({
  labels: props.holdings.map((h) => h.stock.symbol),
  datasets: [
    {
      data: props.holdings.map((h) => h.current_value),
      backgroundColor: props.holdings.map((_, i) => COLORS[i % COLORS.length]),
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

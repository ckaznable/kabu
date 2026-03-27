<script setup lang="ts">
import { computed } from 'vue'
import { Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Tooltip,
} from 'chart.js'
import type { HoldingSummary } from '../api'

ChartJS.register(CategoryScale, LinearScale, BarElement, Tooltip)

const props = defineProps<{ holdings: HoldingSummary[] }>()

const chartData = computed(() => ({
  labels: props.holdings.map((h) => h.stock.symbol),
  datasets: [
    {
      label: 'Return (%)',
      data: props.holdings.map((h) => h.gain_loss_percent),
      backgroundColor: props.holdings.map((h) =>
        h.gain_loss_percent >= 0 ? '#6c8aff' : '#f87171'
      ),
      borderRadius: 4,
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
        label: (ctx: { parsed: { y: number | null } }) =>
          `${(ctx.parsed.y ?? 0).toFixed(2)}%`,
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
        callback: (value: number | string) => `${value}%`,
      },
    },
  },
}
</script>

<template>
  <div class="chart-wrap">
    <Bar :data="chartData" :options="options" />
  </div>
</template>

<style scoped>
.chart-wrap {
  height: 240px;
}
</style>

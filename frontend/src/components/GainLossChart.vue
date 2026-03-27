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
      label: 'Gain / Loss ($)',
      data: props.holdings.map((h) => h.gain_loss),
      backgroundColor: props.holdings.map((h) =>
        h.gain_loss >= 0 ? '#34d399' : '#f87171'
      ),
      borderRadius: 4,
    },
  ],
}))

const options = {
  indexAxis: 'y' as const,
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (ctx: { parsed: { x: number | null } }) =>
          `$${(ctx.parsed.x ?? 0).toLocaleString('en-US', { minimumFractionDigits: 2 })}`,
      },
    },
  },
  scales: {
    x: {
      grid: { color: '#2a2d3a' },
      ticks: { color: '#8b8d98' },
    },
    y: {
      grid: { display: false },
      ticks: { color: '#e4e4e7' },
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

export type {
  Stock,
  CreateStock,
  UpdateStock,
  Price,
  Transaction,
  PortfolioSummary,
  HoldingSummary,
} from './types'

import type { Stock, PortfolioSummary, Transaction } from './types'

export async function fetchPortfolio(): Promise<PortfolioSummary> {
  const res = await fetch('/api/portfolio/summary')
  if (!res.ok) throw new Error('Failed to fetch portfolio')
  return res.json()
}

export async function fetchStocks(): Promise<Stock[]> {
  const res = await fetch('/api/stocks')
  if (!res.ok) throw new Error('Failed to fetch stocks')
  return res.json()
}

export async function createStock(data: {
  symbol: string
  name?: string
  quantity: number
  cost_basis: number
}): Promise<Stock> {
  const res = await fetch('/api/stocks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) throw new Error('Failed to create stock')
  return res.json()
}

export async function updateStock(
  id: number,
  data: { name?: string; quantity: number; cost_basis: number }
): Promise<Stock> {
  const res = await fetch(`/api/stocks/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) throw new Error('Failed to update stock')
  return res.json()
}

export async function deleteStock(id: number): Promise<void> {
  const res = await fetch(`/api/stocks/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error('Failed to delete stock')
}

export async function uploadPdf(file: File): Promise<Transaction[]> {
  const formData = new FormData()
  formData.append('file', file)
  const res = await fetch('/api/pdf/upload', {
    method: 'POST',
    body: formData,
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || 'Failed to upload PDF')
  }
  return res.json()
}

export async function fetchTransactions(): Promise<Transaction[]> {
  const res = await fetch('/api/transactions')
  if (!res.ok) throw new Error('Failed to fetch transactions')
  return res.json()
}

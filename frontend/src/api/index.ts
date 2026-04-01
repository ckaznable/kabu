export type {
  Stock,
  CreateStock,
  UpdateStock,
  UpdateTransaction,
  Price,
  ExchangeRate,
  Transaction,
  PortfolioSummary,
  PortfolioSnapshot,
  HoldingSummary,
} from './types'

import type {
  Stock,
  Price,
  ExchangeRate,
  PortfolioSummary,
  PortfolioSnapshot,
  Transaction,
  UpdateTransaction,
} from './types'

export async function fetchPortfolio(): Promise<PortfolioSummary> {
  const res = await fetch('/api/portfolio/summary')
  if (!res.ok) throw new Error('Failed to fetch portfolio')
  return res.json()
}

export async function fetchPortfolioSnapshots(): Promise<PortfolioSnapshot[]> {
  const res = await fetch('/api/portfolio/snapshots')
  if (!res.ok) throw new Error('Failed to fetch portfolio snapshots')
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
  asset_type?: string
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

export async function uploadPdf(file: File): Promise<void> {
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
}

export async function fetchTransactions(): Promise<Transaction[]> {
  const res = await fetch('/api/transactions')
  if (!res.ok) throw new Error('Failed to fetch transactions')
  return res.json()
}

export async function updateTransaction(
  id: number,
  data: UpdateTransaction
): Promise<Transaction> {
  const res = await fetch(`/api/transactions/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || 'Failed to update transaction')
  }
  return res.json()
}

export async function deleteTransaction(id: number): Promise<void> {
  const res = await fetch(`/api/transactions/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || 'Failed to delete transaction')
  }
}

export async function fetchExchangeRates(): Promise<ExchangeRate[]> {
  const res = await fetch('/api/exchange-rates')
  if (!res.ok) throw new Error('Failed to fetch exchange rates')
  return res.json()
}

export async function fetchPriceHistory(
  symbol: string,
  limit?: number
): Promise<Price[]> {
  const params = limit ? `?limit=${limit}` : ''
  const res = await fetch(`/api/prices/${encodeURIComponent(symbol)}${params}`)
  if (!res.ok) throw new Error('Failed to fetch price history')
  return res.json()
}

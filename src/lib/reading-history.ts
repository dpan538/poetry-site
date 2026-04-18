export interface HistoryEntry {
  id: string
  world: string
  emotionScore: number
  timestamp: number
}

const HISTORY_KEY = 'three-worlds-history'
const MAX_HISTORY = 20

export function getHistory(): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (!raw) return []
    return JSON.parse(raw) as HistoryEntry[]
  } catch {
    return []
  }
}

export function pushHistory(entry: HistoryEntry): void {
  try {
    const history = getHistory()
    const filtered = history.filter(h => h.id !== entry.id)
    filtered.unshift(entry)
    const trimmed = filtered.slice(0, MAX_HISTORY)
    localStorage.setItem(HISTORY_KEY, JSON.stringify(trimmed))
  } catch {
    // localStorage unavailable — fail silently
  }
}

export function getReadingContext(): {
  prev1: { id: string; world: string; emotionScore: number } | null
  prev2: { id: string; world: string; emotionScore: number } | null
} {
  const history = getHistory()
  return {
    prev1: history[0] ?? null,
    prev2: history[1] ?? null,
  }
}

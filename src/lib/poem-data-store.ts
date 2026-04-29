export interface ClientPoemRecord {
  id: string
  world: 'their-world' | 'bless-you' | 'still-life'
  title: string
  firstLine: string
  emotionScore: number
  contentTypes: string[]
  atlasRelations?: Array<{
    id: string
    weight: number
    relation?: string
    reason?: string[]
  }>
}

export const allPoems: ClientPoemRecord[] = []

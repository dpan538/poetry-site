import fs from 'node:fs'
import path from 'node:path'
import atlas from '../src/data/atlas.json' with { type: 'json' }

const root = process.cwd()
const contentRoot = path.join(root, 'src', 'content')
const worlds = ['their-world', 'bless-you', 'still-life']

function listPoemIds() {
  return worlds.flatMap((world) => {
    const dir = path.join(contentRoot, world)
    return fs
      .readdirSync(dir)
      .filter((file) => file.endsWith('.md') || file.endsWith('.mdx'))
      .map((file) => ({
        id: file.replace(/\.(md|mdx)$/u, ''),
        world,
      }))
  })
}

const contentPoems = listPoemIds()
const contentIds = new Set(contentPoems.map((poem) => poem.id))
const atlasIds = new Set(atlas.poems.map((poem) => poem.id))
const errors = []

for (const poem of contentPoems) {
  if (!atlasIds.has(poem.id)) {
    errors.push(`Missing ATLAS record for ${poem.world}/${poem.id}`)
  }
}

for (const poem of atlas.poems) {
  if (!contentIds.has(poem.id)) {
    errors.push(`ATLAS record has no matching content poem: ${poem.id}`)
  }

  for (const relation of poem.related_poems ?? []) {
    if (!contentIds.has(relation.id)) {
      errors.push(`Relation target does not exist: ${poem.id} -> ${relation.id}`)
    }
    if (typeof relation.weight !== 'number' || relation.weight < 0 || relation.weight > 1) {
      errors.push(`Relation weight must be 0..1: ${poem.id} -> ${relation.id}`)
    }
  }
}

if (errors.length) {
  console.error(`ATLAS check failed with ${errors.length} issue(s):`)
  errors.forEach((error) => console.error(`- ${error}`))
  process.exit(1)
}

console.log(`ATLAS check passed: ${contentPoems.length} poems, ${atlas.poems.length} atlas records.`)

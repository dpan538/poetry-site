use crate::{
    content::Work,
    embedding::{Embedder, MODEL, REVISION, cosine, model_dir, normalize},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
    path::Path,
};

#[derive(Deserialize)]
pub struct Scene {
    pub piece: String,
    pub description: String,
}

struct Candidate {
    block: usize,
    text_index: usize,
}

struct Inputs {
    blocks: Range<usize>,
    passages: Vec<Candidate>,
}

#[derive(Serialize)]
pub struct Choice {
    pub scene: usize,
    pub passage: String,
    pub keywords: Vec<String>,
    pub block: usize,
    pub affinity: f32,
    pub scenes: Vec<(String, f32)>,
    pub alternatives: Vec<(usize, String, f32)>,
}

#[derive(Serialize)]
pub struct Study {
    pub model: &'static str,
    pub model_revision: &'static str,
    pub works: usize,
    pub source_spans: usize,
    pub embedded_inputs: usize,
    pub picks: Vec<(String, Choice)>,
}

fn embed_cached(cache: &Path, texts: &[String]) -> Result<Vec<Vec<f32>>> {
    let mut hash = DefaultHasher::new();
    REVISION.hash(&mut hash);
    texts.hash(&mut hash);
    let path = cache.join(format!("vectors-{:016x}.json", hash.finish()));
    if let Ok(file) = fs::File::open(&path) {
        let vectors: Vec<Vec<f32>> = serde_json::from_reader(file)?;
        ensure!(
            vectors.len() == texts.len() && vectors.iter().all(|v| v.len() == 384),
            "invalid embedding cache"
        );
        return Ok(vectors);
    }
    let vectors = Embedder::new(&model_dir()?)?.embed(texts)?;
    let pending = path.with_extension("tmp");
    serde_json::to_writer(fs::File::create(&pending)?, &vectors)?;
    fs::rename(pending, path)?;
    Ok(vectors)
}

fn centroid(vectors: &[Vec<f32>], texts: &[String]) -> Vec<f32> {
    let mut mean = vec![0.0; vectors[0].len()];
    for (vector, text) in vectors.iter().zip(texts) {
        let weight = (text.split_whitespace().count() as f32).sqrt().max(1.0);
        for (dst, value) in mean.iter_mut().zip(vector) {
            *dst += weight * value;
        }
    }
    normalize(&mut mean);
    mean
}

fn address_suffix(word: &str) -> bool {
    matches!(
        word.trim_matches(|c: char| !c.is_alphabetic())
            .to_ascii_lowercase()
            .as_str(),
        "ave"
            | "avenue"
            | "blvd"
            | "boulevard"
            | "ct"
            | "court"
            | "dr"
            | "drive"
            | "ln"
            | "lane"
            | "pkwy"
            | "parkway"
            | "rd"
            | "road"
            | "st"
            | "street"
    )
}

fn prepare(text: &str, preserve_address_context: bool) -> String {
    let words: Vec<_> = text
        .lines()
        .filter(|line| {
            !line
                .trim()
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        })
        .flat_map(str::split_whitespace)
        .collect();
    let mut out = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        if word.len() <= 5 && word.bytes().all(|b| b.is_ascii_digit()) {
            let suffix = words[i + 1..]
                .iter()
                .take(4)
                .position(|next| address_suffix(next));
            if let Some(suffix) = suffix.filter(|&n| n > 0) {
                if preserve_address_context {
                    out.push("street address");
                }
                i += suffix + 2;
                continue;
            }
        }
        out.push(word);
        i += 1;
    }
    out.join(" ").replace('’', "'").replace(['–', '—'], " ")
}

fn key_words(text: &str) -> Vec<String> {
    const STOPS: &str = "about across after again almost along already also among another anything around away back became because become been before being below both came cannot could didn't doesn’t don't down during each either enough even ever every first following found from gave goes going gone good greater hadn’t happened have having here herself himself inside into isn't itself just kept kind knew know known last later least less like little look looked looking made make many may might mine more most much must near need never next nobody none nothing often once only onto other ought ours outside over past perhaps please quite rather really said same saying says seem seemed seen seems shall should since some someone something sometimes still such sure taken taking telling than that their theirs them themselves then there these they thing things think this those though thought through thus till together toward towards tried under until upon used uses very want wants wasn't wasn’t well went were what whatever when where whether which while whom whose will with within without won't wouldn’t would your yours yourself";
    prepare(text, false)
        .split(|c: char| !c.is_alphabetic() && !matches!(c, '-' | '\''))
        .map(|s| s.trim_matches(['-', '’', '\'']).to_lowercase())
        .filter(|s| {
            s.chars().count() >= 3
                && s.chars().count() < 26
                && !s.contains(['’', '\''])
                && !matches!(
                    s.as_str(),
                    "ave" | "blvd" | "pkwy" | "etc" | "mrs" | "prof" | "inc" | "ltd" | "apt"
                )
                && !STOPS.split_whitespace().any(|w| w == s)
        })
        .collect()
}

// Allocate the most distinctive matches first. Reuse costs discourage repetition
// without discarding a considerably better semantic match.
fn assign_scenes(
    works: &[Work],
    rankings: &[Vec<(usize, f32)>],
    scenes: usize,
    mut chosen: Vec<Option<usize>>,
) -> Vec<usize> {
    let mut volumes: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, work) in works.iter().enumerate() {
        let volume = work.slug.split('/').next().unwrap_or_default();
        volumes.entry(volume).or_default().push(i);
    }
    let mut used = vec![0; scenes];
    for &scene in chosen.iter().flatten() {
        used[scene] += 1;
    }
    for volume in volumes.values_mut() {
        volume.sort_by_key(|&i| works[i].order);
        let mut local = vec![0; scenes];
        for &work in volume.iter() {
            if let Some(scene) = chosen[work] {
                local[scene] += 1;
            }
        }
        let unassigned = volume
            .iter()
            .filter(|&&work| chosen[work].is_none())
            .count();
        for _ in 0..unassigned {
            let mut best: Option<(usize, usize, f32)> = None;
            for (position, &work) in volume.iter().enumerate() {
                if chosen[work].is_some() {
                    continue;
                }
                let left = position.checked_sub(1).and_then(|i| chosen[volume[i]]);
                let right = volume.get(position + 1).and_then(|&i| chosen[i]);
                let mut available: Vec<_> = rankings[work]
                    .iter()
                    .filter(|&&(scene, _)| Some(scene) != left && Some(scene) != right)
                    .map(|&(scene, score)| {
                        (
                            scene,
                            score - 0.043 * local[scene] as f32 - 0.012 * used[scene] as f32,
                        )
                    })
                    .collect();
                available.sort_by(|a, b| b.1.total_cmp(&a.1));
                let (scene, score) = available[0];
                let margin = available
                    .get(1)
                    .map_or(f32::INFINITY, |next| score - next.1);
                if best.is_none_or(|(_, _, widest)| margin > widest) {
                    best = Some((work, scene, margin));
                }
            }
            let (work, scene, _) = best.expect("unassigned work has a scene candidate");
            chosen[work] = Some(scene);
            local[scene] += 1;
            used[scene] += 1;
        }
    }
    chosen
        .into_iter()
        .map(|scene| scene.expect("all works assigned"))
        .collect()
}

pub fn rank(
    works: &[Work],
    catalog: &[Scene],
    cache: &Path,
    established: Option<&BTreeMap<String, String>>,
) -> Result<Study> {
    ensure!(catalog.len() >= 3, "MANIFOLD needs at least three scenes");
    let scene_index = |piece: &str| {
        catalog
            .iter()
            .position(|scene| scene.piece == piece)
            .with_context(|| format!("Unknown MANIFOLD scene {piece}"))
    };
    let fixed = works
        .iter()
        .map(|work| work.scene.as_deref().map(&scene_index).transpose())
        .collect::<Result<Vec<_>>>()?;
    fs::create_dir_all(cache)?;
    let mut texts = Vec::new();
    let mut inputs = Vec::new();
    for work in works {
        let start = texts.len();
        for block in &work.blocks {
            // A non-overlapping pass covers every word, including section labels.
            let words: Vec<_> = block.split_whitespace().collect();
            texts.extend(words.chunks(75).map(|chunk| chunk.join(" ")));
        }
        let blocks = start..texts.len();
        let mut passages = Vec::new();
        for (block_index, block) in work.blocks.iter().enumerate() {
            let sentences: Vec<_> = block
                .split_inclusive(['.', '?', '!'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let lines: Vec<_> = if work.prose {
                sentences
            } else {
                block.lines().collect()
            };
            for start in 0..lines.len() {
                for end in start + 1..=(start + 2).min(lines.len()) {
                    let passage = if work.prose {
                        let left = lines[start].as_ptr() as usize - block.as_ptr() as usize;
                        let right = lines[end - 1].as_ptr() as usize - block.as_ptr() as usize
                            + lines[end - 1].len();
                        block[left..right].to_owned()
                    } else {
                        lines[start..end].join("\n")
                    };
                    let length = passage.split_whitespace().count();
                    if !(3..=40).contains(&length) {
                        continue;
                    }
                    passages.push(Candidate {
                        block: block_index,
                        text_index: texts.len(),
                    });
                    texts.push(passage);
                }
            }
        }
        if passages.is_empty() {
            passages.push(Candidate {
                block: 0,
                text_index: texts.len(),
            });
            texts.push(work.blocks[0].clone());
        }
        inputs.push(Inputs { blocks, passages });
    }
    let sources = inputs.iter().map(|w| w.blocks.len()).sum();
    let scene_start = texts.len();
    texts.extend(catalog.iter().map(|scene| scene.description.clone()));
    let model_inputs: Vec<_> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            if i < scene_start {
                prepare(text, true)
            } else {
                text.clone()
            }
        })
        .collect();
    let vectors = embed_cached(cache, &model_inputs)?;
    let centroids: Vec<_> = inputs
        .iter()
        .map(|i| centroid(&vectors[i.blocks.clone()], &texts[i.blocks.clone()]))
        .collect();
    let anchors = &vectors[scene_start..];
    let words_by_work: Vec<BTreeSet<String>> = works
        .iter()
        .map(|w| w.blocks.iter().flat_map(|b| key_words(b)).collect())
        .collect();
    let mut frequencies = BTreeMap::new();
    for words in &words_by_work {
        for word in words {
            *frequencies.entry(word.clone()).or_insert(0usize) += 1;
        }
    }
    let vocabulary: Vec<_> = frequencies.keys().cloned().collect();
    let word_vectors = embed_cached(cache, &vocabulary)?;
    let rankings: Vec<Vec<_>> = inputs
        .iter()
        .enumerate()
        .map(|(work_index, input)| {
            let mean = &centroids[work_index];
            let mut scores: Vec<_> = anchors
                .iter()
                .enumerate()
                .map(|(i, scene)| {
                    let mut affinities: Vec<_> = vectors[input.blocks.clone()]
                        .iter()
                        .map(|span| cosine(span, scene))
                        .collect();
                    affinities.sort_by(|a, b| b.total_cmp(a));
                    let top = &affinities[..affinities.len().min(3)];
                    let coverage = top.iter().sum::<f32>() / top.len() as f32;
                    let background = centroids
                        .iter()
                        .enumerate()
                        .filter(|(idx, _)| *idx != work_index)
                        .map(|(_, c)| cosine(c, scene))
                        .sum::<f32>()
                        / (centroids.len() - 1).max(1) as f32;
                    (
                        i,
                        0.63 * coverage + 0.37 * cosine(mean, scene) - 0.21 * background,
                    )
                })
                .collect();
            scores.sort_by(|a, b| b.1.total_cmp(&a.1));
            scores
        })
        .collect();
    let assigned = if let Some(established) = established {
        works
            .iter()
            .zip(fixed)
            .map(|(work, pin)| {
                if let Some(scene) = pin {
                    return Ok(scene);
                }
                let piece = established
                    .get(&work.slug)
                    .with_context(|| format!("Missing current scene for {}", work.slug))?;
                scene_index(piece)
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        assign_scenes(works, &rankings, catalog.len(), fixed)
    };
    let mut picks = Vec::new();
    for (work_index, (work, input)) in works.iter().zip(&inputs).enumerate() {
        let mean = &centroids[work_index];
        let scenes = &rankings[work_index];
        let scene = assigned[work_index];
        let affinity = scenes.iter().find(|(i, _)| *i == scene).unwrap().1;
        let mut candidates: Vec<_> = input
            .passages
            .iter()
            .map(|candidate| {
                let vector = &vectors[candidate.text_index];
                let length = texts[candidate.text_index].split_whitespace().count();
                let brevity = if work.prose { 24 } else { 14 };
                let score = 0.49 * cosine(vector, mean) + 0.51 * cosine(vector, &anchors[scene])
                    - 0.0017 * (length as i32 - brevity).unsigned_abs() as f32;
                (candidate, score)
            })
            .collect();
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let best = candidates[0].0;
        let passage_vector = &vectors[best.text_index];
        let passage_terms: BTreeSet<_> = key_words(&texts[best.text_index]).into_iter().collect();
        let mut scored: Vec<_> = words_by_work[work_index]
            .iter()
            .filter_map(|word| {
                let index = vocabulary.binary_search(word).ok()?;
                let vector = &word_vectors[index];
                let idf = ((1 + works.len()) as f32 / (1 + frequencies[word]) as f32).ln();
                let local = if passage_terms.contains(word) {
                    0.065
                } else {
                    0.0
                };
                let score = 0.43 * cosine(vector, &anchors[scene])
                    + 0.33 * cosine(vector, mean)
                    + 0.24 * cosine(vector, passage_vector)
                    + 0.012 * idf
                    + local;
                Some((word, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut keywords: Vec<String> = Vec::new();
        for (word, _) in scored {
            if keywords
                .iter()
                .any(|selected| selected.trim_end_matches('s') == word.trim_end_matches('s'))
            {
                continue;
            }
            keywords.push(word.clone());
            if keywords.len() == 3 {
                break;
            }
        }
        ensure!(keywords.len() == 3, "too few keywords for {}", work.slug);
        picks.push((
            work.slug.clone(),
            Choice {
                scene,
                passage: texts[best.text_index].clone(),
                keywords,
                block: best.block + 1,
                affinity,
                scenes: scenes
                    .iter()
                    .map(|&(i, score)| (catalog[i].piece.clone(), score))
                    .collect(),
                alternatives: candidates
                    .iter()
                    .take(5)
                    .map(|(c, score)| (c.block + 1, texts[c.text_index].clone(), *score))
                    .collect(),
            },
        ));
    }
    Ok(Study {
        model: MODEL,
        model_revision: REVISION,
        works: works.len(),
        source_spans: sources,
        embedded_inputs: texts.len() + vocabulary.len(),
        picks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_context_but_never_a_keyword() {
        let passage = "(21)\nCrown Antiques,\nAt 1018 Fifth Ave,\nGlass catches light on the road.";
        assert_eq!(
            prepare(passage, true),
            "Crown Antiques, At street address Glass catches light on the road."
        );
        let words = key_words(passage);
        assert!(words.contains(&"glass".into()));
        assert!(words.contains(&"road".into()));
        assert!(!words.contains(&"ave".into()));
        assert!(!words.contains(&"fifth".into()));
    }

    #[test]
    fn scene_allocation_allows_strong_reuse_but_not_on_neighbours() {
        let works: Vec<_> = (1..=4)
            .map(|order| Work {
                slug: format!("volume/work-{order}"),
                blocks: vec![],
                prose: false,
                order,
                scene: None,
            })
            .collect();
        let scores = vec![
            vec![(0, 0.8), (1, 0.1), (2, 0.0), (3, 0.0)],
            vec![(1, 0.8), (0, 0.1), (2, 0.0), (3, 0.0)],
            vec![(0, 0.7), (1, 0.1), (2, 0.0), (3, 0.0)],
            vec![(0, 0.5), (2, 0.45), (3, 0.44), (1, 0.0)],
        ];
        assert_eq!(
            assign_scenes(&works, &scores, 4, vec![None; 4]),
            vec![0, 1, 0, 2]
        );
        let pinned = assign_scenes(&works, &scores, 4, vec![None, Some(0), None, None]);
        assert_eq!(pinned[1], 0);
        assert_ne!(pinned[0], 0);
        assert_ne!(pinned[2], 0);
    }
}

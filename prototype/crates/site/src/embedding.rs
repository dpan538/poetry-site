use anyhow::{Context, Result, anyhow, ensure};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokenizers::{Encoding, Tokenizer};

pub const REVISION: &str = "1110a243fdf4706b3f48f1d95db1a4f5529b4d41";
pub const MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";

pub fn model_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("MANIFOLD_MODEL_DIR") {
        return Ok(path.into());
    }
    let cache = std::env::var_os("HF_HUB_CACHE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HF_HOME").map(|p| PathBuf::from(p).join("hub")))
        .or_else(|| {
            std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache/huggingface/hub"))
        })
        .context("set MANIFOLD_MODEL_DIR to the local model directory")?;
    Ok(cache
        .join("models--sentence-transformers--all-MiniLM-L6-v2")
        .join("snapshots")
        .join(REVISION))
}

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
}

impl Embedder {
    pub fn new(dir: &Path) -> Result<Self> {
        let config: Config = serde_json::from_slice(
            &fs::read(dir.join("config.json"))
                .with_context(|| format!("model files missing in {}", dir.display()))?,
        )?;
        let mut tokenizer =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow!("{e}"))?;
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(None)
            .map_err(|e| anyhow!("{e}"))?;
        let weights = fs::read(dir.join("model.safetensors"))?;
        let variables = VarBuilder::from_buffered_safetensors(weights, DType::F32, &Device::Cpu)?;
        Ok(Self {
            model: BertModel::load(variables, &config)?,
            tokenizer,
        })
    }

    fn encode(&self, text: &str) -> Result<Encoding> {
        self.tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("{e}"))
    }

    pub fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut inputs = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let encoding = self.encode(text)?;
                ensure!(
                    encoding.len() <= 256,
                    "embedding input {i} exceeds 256 tokens"
                );
                Ok((i, encoding))
            })
            .collect::<Result<Vec<_>>>()?;
        inputs.sort_by_key(|(_, row)| row.len());
        let mut embeddings = vec![Vec::new(); inputs.len()];
        for (batch_index, batch) in inputs.chunks(16).enumerate() {
            let width = batch.last().context("empty embedding batch")?.1.len();
            let mut ids = vec![0_u32; batch.len() * width];
            let mut mask = vec![0_u32; ids.len()];
            for (i, (_, row)) in batch.iter().enumerate() {
                ids[i * width..i * width + row.len()].copy_from_slice(row.get_ids());
                mask[i * width..i * width + row.len()].fill(1);
            }
            let shape = (batch.len(), width);
            let ids = Tensor::from_vec(ids, shape, &Device::Cpu)?;
            let mask = Tensor::from_vec(mask, shape, &Device::Cpu)?;
            let tokens = self.model.forward(&ids, &ids.zeros_like()?, Some(&mask))?;
            let mask = mask.to_dtype(DType::F32)?.unsqueeze(2)?;
            let average = tokens
                .broadcast_mul(&mask)?
                .sum(1)?
                .broadcast_div(&mask.sum(1)?)?;
            for ((i, _), mut vector) in batch.iter().zip(average.to_vec2::<f32>()?) {
                normalize(&mut vector);
                embeddings[*i] = vector;
            }
            if batch_index.is_multiple_of(10) {
                eprintln!(
                    "embedded {}/{} text spans",
                    ((batch_index + 1) * 16).min(inputs.len()),
                    inputs.len()
                );
            }
        }
        Ok(embeddings)
    }
}

pub fn normalize(vector: &mut [f32]) {
    let norm = vector
        .iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt()
        .max(f32::EPSILON);
    vector.iter_mut().for_each(|x| *x /= norm);
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

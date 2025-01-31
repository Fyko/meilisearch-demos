use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use color_eyre::{Result, eyre::eyre};

use config::CONFIG;
use flate2::bufread::GzDecoder;
use futures::{channel::mpsc};
use futures::stream::StreamExt;
use futures_timer::Delay;

use cargo_toml::Manifest;
use meilisearch_sdk::Client;
use serde::{Deserialize, Serialize};
use tar::Archive;

pub mod config;
pub mod backoff;

#[derive(Debug, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub vers: String,
}

#[derive(Debug, Serialize)]
pub struct CompleteCrateInfos {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub readme: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadsCrateInfos {
    pub name: String,
    downloads: u64,
}

pub async fn retrieve_crate_toml(info: &CrateInfo) -> Result<CompleteCrateInfos> {
    let url = format!(
        "https://static.crates.io/crates/{name}/{name}-{version}.crate",
        name = info.name,
        version = info.vers,
    );

    let mut result = None;
    for multiplier in backoff::new().take(10) {
        match reqwest::get(&url).await {
            Ok(res) => {
                result = Some(res);
                break;
            }
            Err(e) => {
                let dur = Duration::from_secs(1) * (multiplier + 1);
                tracing::warn!("error downloading {} {} retrying in {:.2?}", url, e, dur);
                let _ = Delay::new(dur).await;
            }
        }
    }

    let res = match result {
        Some(res) => res,
        None => return Err(color_eyre::eyre::eyre!("Could not download {}", url)),
    };

    if !res.status().is_success() {
        let body = res.text().await?;
        return Err(eyre!("Could not download {}: {}", url, body));
    }

    let bytes = res.bytes().await?;
    let archive = GzDecoder::new(&bytes[..]);
    let mut tar = Archive::new(archive);

    let mut toml_infos = None;
    let mut readme = None;

    for res in tar.entries()? {
        // stop early if we found both files
        if toml_infos.is_some() && readme.is_some() {
            break;
        }

        let mut entry = res?;
        let path = entry.path()?;
        let file_name = path.file_name().and_then(|s| s.to_str());

        match file_name {
            Some("Cargo.toml") if toml_infos.is_none() => {
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;

                let manifest = Manifest::from_slice(&content)?;
                let package = match manifest.package {
                    Some(package) => package,
                    None => break,
                };

                let name = info.name.clone();
                let description = package.description.unwrap_or_default();
                let keywords = package.keywords;
                let categories = package.categories;
                let version = info.vers.clone();

                toml_infos = Some((name, description, keywords, categories, version));
            }
            Some("README.md") if readme.is_none() => {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;

                let options = comrak::ComrakOptions::default();
                let html = comrak::markdown_to_html(&content, &options);

                let document = scraper::Html::parse_document(&html);
                let html = document.root_element();
                let text = html.text().collect();

                readme = Some(text);
            }
            _ => (),
        }
    }

    match (toml_infos, readme) {
        (Some((name, description, keywords, categories, version)), readme) => {
            Ok(CompleteCrateInfos {
                name,
                description: description.get().unwrap_or(&"".to_string()).to_string(),
                keywords: keywords.get().unwrap_or(&vec![]).to_vec(),
                categories: categories.get().unwrap_or(&vec![]).to_vec(),
                readme: readme.unwrap_or_default(),
                version,
            })
        }
        (None, _) => Err(eyre!("No Cargo.toml found in this crate")),
    }
}

// something in here is causing a rucus
pub async fn chunk_complete_crates_info_to_meili(
    client: Arc<Client>,
    receiver: mpsc::Receiver<CompleteCrateInfos>,
) -> Result<()> {
    let index = client.index(CONFIG.meili_index_uid.clone());

    let mut chunk_count = 0;
    let mut receiver = receiver.chunks(100);
    while let Some(chunk) = StreamExt::next(&mut receiver).await {
        tracing::debug!("chunk {chunk_count} len: {}", chunk.len());
        chunk_count += 1;

        let task = index.add_or_update(&chunk, Some("name")).await?;
        let res = client.wait_for_task(task, None, None).await?;
        tracing::debug!("{res:#?}");
    }

    Ok(())
}

pub async fn chunk_downloads_crates_info_to_meili(
    client: Arc<Client>,
    receiver: mpsc::Receiver<DownloadsCrateInfos>,
) -> Result<()> {
    let index = client.index(CONFIG.meili_index_uid.clone());

    let mut receiver = receiver.chunks(150);
    while let Some(chunk) = StreamExt::next(&mut receiver).await {
        let task = index.add_or_update(&chunk, Some("name")).await?;
        let res = client.wait_for_task(task, None, None).await?;
        tracing::info!("{res:#?}");
    }

    Ok(())
}

pub fn create_meilisearch_client() -> Arc<Client> {
    Arc::new(Client::new(CONFIG.meili_host_url.clone(), Some(CONFIG.meili_api_key.clone())))
}

pub fn init_logging() {
    color_eyre::install().expect("failed to install color_eyre");

    use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

    Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,html5ever=info,isahc=info".into()))
        .with(fmt::layer())
        .init();
}

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use reqwest::header;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_aux::prelude::*;

use crate::prelude::*;

use super::{IterAdapter, Reqwest, Wrapper};

#[derive(Debug)]
struct Payload<I> {
    // #[serde(rename = "itemcount")]
    // #[serde(rename = "publishedfileids")]
    file_id: I,
}

impl<I> Payload<I> {
    fn new(file_id: I) -> Self {
        Self { file_id }
    }
}

impl<I: Iterator<Item = FileId> + Clone> Serialize for Payload<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let adapter = IterAdapter {
            iter: self.file_id.clone(),
            len: 0.into(),
        };
        let mut state = serializer.serialize_struct("Payload", 2)?;
        state.serialize_field("publishedfileids", &adapter)?;
        state.serialize_field("itemcount", &adapter.len.get())?;
        state.end()
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "resultcount")]
    pub count: usize,
    #[serde(rename = "publishedfiledetails")]
    pub details: Vec<Detail>,
}

#[derive(Debug, Deserialize)]
pub struct Detail {
    #[serde(
        rename = "publishedfileid",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub file_id: FileId,
    pub result: u32,

    #[serde(flatten)]
    pub inner: Option<DetailInner>,
}

impl Detail {
    pub fn is_valid_item(&self) -> bool {
        self.result == 1 && self.inner.is_some()
    }
}

#[derive(Debug, Deserialize)]
pub struct DetailInner {
    #[serde(
        rename = "consumer_app_id",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub app_id: FileId,
    pub title: String,
    pub description: String,
    #[serde(with = "ts_seconds")]
    pub time_created: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub time_updated: DateTime<Utc>,
}

const URL: &str = "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/?";

pub async fn call(file_ids: impl Iterator<Item = u64> + Clone) -> Result<Response> {
    let payload = Payload::new(file_ids);
    let payload = serde_qs::to_string(&payload)?;

    let client = Reqwest::client().await;
    let response = client
        .post(URL)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(payload)
        .send()
        .await?;

    let Wrapper { response } = response.json::<Wrapper<Response>>().await?;
    Ok(response)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test() {
        color_eyre::install().unwrap();
        let resp = call([2824342092, 2529002857, 1111].into_iter())
            .await
            .unwrap();
        println!("{:#?}", resp);
    }
}

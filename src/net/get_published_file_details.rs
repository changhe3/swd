use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

use crate::prelude::*;

use super::{Reqwest, Wrapper};

#[derive(Debug, Serialize)]
struct Payload<'a> {
    #[serde(rename = "itemcount")]
    count: u32,
    #[serde(rename = "publishedfileids")]
    file_id: &'a [FileId],
}

impl<'a> Payload<'a> {
    fn new(file_id: &'a [FileId]) -> Self {
        let count = file_id.len() as u32;
        Self { count, file_id }
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
    file_id: FileId,
    result: u32,

    #[serde(flatten)]
    inner: Option<DetailInner>,
}

impl Detail {
    pub fn is_valid_item(&self) -> bool {
        self.result == 1 && self.inner.is_some()
    }
}

#[derive(Debug, Deserialize)]
pub struct DetailInner {
    title: String,
    description: String,
    #[serde(with = "ts_seconds")]
    time_created: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    time_updated: DateTime<Utc>,
}

const URL: &str = "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/?";

pub async fn call(file_ids: &[FileId]) -> Result<Response> {
    let payload = Payload::new(file_ids);
    let payload = serde_qs::to_string(&payload)?;

    let client = Reqwest::client().await;
    let response = client
        .post(URL)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(payload)
        .send()
        .await?;

    let Wrapper { response } = dbg!(response).json::<Wrapper<Response>>().await?;
    Ok(response)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test() {
        color_eyre::install().unwrap();
        let resp = call(&[2824342092, 2529002857, 1111]).await.unwrap();
        println!("{:#?}", resp);
    }
}

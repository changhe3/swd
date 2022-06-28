use reqwest::header;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::prelude::*;

use super::{IterAdapter, Reqwest, Wrapper};
use serde_aux::prelude::*;

#[derive(Debug)]
struct Payload<I> {
    // #[serde(rename = "collectioncount")]
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
        state.serialize_field("collectioncount", &adapter.len.get())?;
        state.end()
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "collectiondetails")]
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
    pub children: Option<Vec<Child>>,
}

#[derive(Debug, Deserialize)]
pub struct Child {
    #[serde(
        rename = "publishedfileid",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub file_id: FileId,
    pub sortorder: u32,
    pub filetype: u32,
}

const URL: &str = r"https://api.steampowered.com/ISteamRemoteStorage/GetCollectionDetails/v1/?";

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
    use super::call;

    #[tokio::test]
    async fn test() {
        color_eyre::install().unwrap();
        let resp = call([1626860092, 2529002857].into_iter()).await.unwrap();
        println!("{:#?}", resp);
    }
}

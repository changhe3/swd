use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

use super::{Reqwest, Wrapper};
use serde_aux::prelude::*;

#[derive(Debug, Serialize)]
struct Payload<'a> {
    #[serde(rename = "collectioncount")]
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

    let Wrapper { response } = response.json::<Wrapper<Response>>().await?;
    Ok(response)

    // Err(color_eyre::eyre::eyre!("err"))
}

#[cfg(test)]
mod tests {
    use super::call;

    #[tokio::test]
    async fn test() {
        color_eyre::install().unwrap();
        let resp = call(&[1626860092, 2529002857]).await.unwrap();
        println!("{:#?}", resp);
    }
}

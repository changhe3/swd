mod net;
mod prelude;
mod util;

use crate::prelude::*;
use chrono::{DateTime, Utc};
use net::{
    get_collection_details::{self, Detail as CollectionDetail},
    get_published_file_details::{self, Detail as FileDetail, DetailInner},
};
use std::{collections::HashMap, fmt::Debug};
use structopt::StructOpt;
use tokio::{
    io::{stdout, AsyncWriteExt},
    main,
};

#[derive(Debug, StructOpt)]
#[structopt(about = "Download workshop item and collections from steam workshop")]
struct Params {
    #[structopt(
        short,
        long,
        help = "Execute the produced command through steamcmd, otherwise the command is only printed to stdout"
    )]
    exec: bool,

    #[structopt(short, long, help = "Review each mod one by one")]
    review: bool,

    #[structopt(short, long, default_value = "anonymous")]
    username: String,

    #[structopt(
        long,
        takes_value(true),
        require_equals(true),
        possible_values(&["simple", "csv"]),
        help = "Save the mod orders of collections to specified format to the current working directory"
    )]
    save: Option<String>,

    #[structopt(help = "File IDs of the mods and collections to download")]
    files: Vec<FileId>,
}

#[derive(Debug)]
struct WFile {
    file_id: FileId,
    children: Option<Vec<FileId>>,
    title: String,
    description: String,
    time_created: DateTime<Utc>,
    time_updated: DateTime<Utc>,
}

impl WFile {
    fn is_collection(&self) -> bool {
        self.children.is_some()
    }

    async fn retrieve_details(&mut self) {
        todo!()
    }

    async fn save_order() {
        todo!()
    }
}

#[derive(Debug)]
struct WFiles {
    params: Params,
    all_files: HashMap<FileId, WFile>,
}

impl WFiles {
    async fn new(params: Params) -> Result<Self> {
        let mut c_details = get_collection_details::call(params.files.iter().copied()).await?;

        let all_ids = c_details.details.values().flat_map(|detail| {
            let inner = detail.children.as_ref().into_iter().flat_map(|v| {
                v.iter()
                    .filter_map(|c| (c.filetype == 0).then(|| c.file_id))
            });
            std::iter::once(detail.file_id).chain(inner)
        });
        let f_details = get_published_file_details::call(all_ids).await?;

        let all_files = f_details
            .details
            .into_iter()
            .filter_map(|d| {
                if d.result == 1 {
                    let DetailInner {
                        title,
                        description,
                        time_created,
                        time_updated,
                    } = d.inner.unwrap();

                    let file_id = d.file_id;
                    let children = {
                        let children = &mut c_details.details.get_mut(&file_id).unwrap().children;
                        children.as_mut().map(|children| {
                            children.sort_by_key(|c| c.sortorder);
                            children
                                .iter()
                                .filter_map(|c| (c.filetype == 0).then(|| c.file_id))
                                .collect::<Vec<_>>()
                        })
                    };

                    let wfile = WFile {
                        file_id,
                        children,
                        title,
                        description,
                        time_created,
                        time_updated,
                    };
                    Some((file_id, wfile))
                } else {
                    Self::invalid_id(d.file_id);
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        Ok(Self { params, all_files })
    }

    fn invalid_id(id: FileId) {
        println!("Invalid File ID: {}", id);
    }
}

#[main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let params = Params::from_args();
    println!("Hello, world!");
    Ok(())
}

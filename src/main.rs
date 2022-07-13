mod net;
mod prelude;
mod util;

use crate::prelude::*;
use chrono::{DateTime, Utc};
use dialoguer::Input;
use itertools::Itertools;
use net::{
    get_collection_details,
    get_published_file_details::{self, DetailInner},
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::File,
    process::Command,
    str::FromStr,
};
use std::{env::current_dir, fmt::Write as FmtWrite, io::Write as IoWrite};
use structopt::StructOpt;

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
    app_id: FileId,
    children: Option<Vec<FileId>>,
    title: String,
    description: String,
    time_created: DateTime<Utc>,
    time_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewOptions {
    Yes,
    No,
    Skip,
}

impl FromStr for ReviewOptions {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "y" | "yes" => Ok(Self::Yes),
            "n" | "no" => Ok(Self::No),
            "skip" => Ok(Self::Skip),
            _ => Err(color_eyre::eyre::eyre!("invalid input")),
        }
    }
}

impl ToString for ReviewOptions {
    fn to_string(&self) -> String {
        match self {
            ReviewOptions::Yes => "yes",
            ReviewOptions::No => "no",
            ReviewOptions::Skip => "skip",
        }
        .into()
    }
}

impl WFile {
    fn is_collection(&self) -> bool {
        self.children.is_some()
    }

    fn prompt(&self, review: bool) -> Result<ReviewOptions> {
        let item = if self.is_collection() {
            "collection"
        } else {
            "mod"
        };
        println!(
            "Found {} \t\t {}: \t {} \t\t Created at: \t {} \t\t Updated at: \t {} ",
            item, self.file_id, self.title, self.time_created, self.time_updated
        );
        let res = if review {
            let mut input = Input::<ReviewOptions>::new();
            input.with_prompt(format!("Description: {}", self.description));
            input.default(ReviewOptions::Yes);
            input.interact_text()?
        } else {
            ReviewOptions::Yes
        };
        Ok(res)
    }
}

#[derive(Debug)]
struct WFiles {
    params: Params,
    all_files: HashMap<FileId, WFile>,
}

impl WFiles {
    fn new(mut params: Params) -> Result<Self> {
        // retieve the mod ids of each collection listed in params.files
        let mut c_details = get_collection_details::call(params.files.iter().copied())?;

        // retrieve details about all mods and collections listed in params.files, plus all submods inside each collection
        let all_ids = c_details.details.values().flat_map(|detail| {
            let inner = detail.children.iter().flat_map(|v| {
                // remove linked collections
                v.iter()
                    .filter_map(|c| (c.filetype == 0).then_some(c.file_id))
            });
            std::iter::once(detail.file_id).chain(inner).dedup()
        });
        let f_details = get_published_file_details::call(all_ids)?;

        let all_files = f_details
            .details
            .into_iter()
            .filter_map(|d| {
                if !d.is_valid_item() {
                    Self::invalid_id(d.file_id);
                    return None;
                }

                let DetailInner {
                    app_id,
                    title,
                    description,
                    time_created,
                    time_updated,
                } = d.inner.unwrap(); // d.inner is guaranteed non-null since d.result == 1

                let file_id = d.file_id;
                let children = {
                    let children = c_details
                        .details
                        .get_mut(&file_id)
                        .and_then(|d| d.children.as_mut());
                    children.map(|children| {
                        children.sort_by_key(|c| c.sortorder);
                        children
                            .iter()
                            .filter_map(|c| (c.filetype == 0).then(|| c.file_id))
                            .collect::<Vec<_>>()
                    })
                };

                let wfile = WFile {
                    file_id,
                    app_id,
                    children,
                    title,
                    description,
                    time_created,
                    time_updated,
                };
                Some((file_id, wfile))
            })
            .collect::<HashMap<_, _>>();

        // Remove invalid mod ids
        params.files.retain(|id| all_files.contains_key(id));
        Ok(Self { params, all_files })
    }

    fn invalid_id(id: FileId) {
        println!("Invalid File ID: {}", id);
    }

    fn build_cmd(&self) -> Result<String> {
        let mut cmd = format!("steamcmd +login {} ", self.params.username);
        let wd = current_dir().unwrap();
        let mut all_mods = HashSet::new();

        for file_id in self.params.files.iter() {
            let file = &self.all_files[file_id];
            if ReviewOptions::Yes != file.prompt(self.params.review)? {
                continue;
            }

            if !file.is_collection() {
                if all_mods.insert(file_id) {
                    write!(
                        cmd,
                        "+workshop_download_item {} {} ",
                        file.app_id, file.file_id
                    )?;
                }
            } else {
                let save_path = self
                    .params
                    .save
                    .as_ref()
                    .map(|_| wd.join(format!("{}_{}.csv", file_id, file.title)));

                let mut save_file = save_path.map(File::create).transpose()?;

                for file_id in file.children.as_ref().unwrap().iter() {
                    let inner_file = &self.all_files[file_id];
                    match inner_file.prompt(self.params.review)? {
                        ReviewOptions::Yes => {
                            if all_mods.insert(&inner_file.file_id) {
                                write!(
                                    cmd,
                                    "+workshop_download_item {} {} ",
                                    inner_file.app_id, inner_file.file_id
                                )?;
                            }

                            if let Some(save_file) = save_file.as_mut() {
                                match self.params.save.as_deref().unwrap() {
                                    "simple" => writeln!(save_file, "{}", file_id)?,
                                    "csv" => writeln!(
                                        save_file,
                                        "{}\t{}\t{}\t{}",
                                        inner_file.file_id,
                                        inner_file.title,
                                        inner_file.time_created,
                                        inner_file.time_updated
                                    )?,
                                    _ => unreachable!(),
                                };
                            }
                        }
                        ReviewOptions::No => {
                            continue;
                        }
                        ReviewOptions::Skip => {
                            break;
                        }
                    }
                }

                if let Some(save_file) = save_file.as_mut() {
                    save_file.flush()?;
                }
            }
        }

        cmd.push_str("+quit ");
        Ok(cmd.replace('\n', ""))
    }

    fn run(&self) -> Result<()> {
        let cmd = self.build_cmd()?;
        if !self.params.exec {
            println!("\n{}", cmd);
        } else {
            let mut cmd = cmd.split_ascii_whitespace();
            if let Ok(mut proc) = Command::new(cmd.next().unwrap()).args(cmd).spawn() {
                proc.wait().unwrap();
            } else {
                return Err(color_eyre::eyre::eyre!("steamcmd failed"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() -> Result<()> {
        let params = Params::from_iter(["swd"].into_iter());
        __main__(params)
    }

    #[test]
    fn test_collection() -> Result<()> {
        let params = Params::from_iter(["swd", "368330611"].into_iter());
        __main__(params)
    }

    #[test]
    fn test_collection_review() -> Result<()> {
        let params = Params::from_iter(["swd", "-r", "368330611", "116676096"].into_iter());
        __main__(params)
    }
}

fn __main__(params: Params) -> Result<()> {
    color_eyre::install()?;
    if params.files.is_empty() {
        Params::clap().print_long_help()?;
    } else {
        let wfiles = WFiles::new(params)?;
        wfiles.run()?;
    }

    Ok(())
}

fn main() -> Result<()> {
    __main__(Params::from_args())
}

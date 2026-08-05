use std::{fs, path::Path, process::exit};

use anyhow::Result;

use crate::{
    package_manager::{web::create::upload_file},
};

pub async fn upload_files(
    paths: Vec<String>,
    package_source: &str,
    registry_id: i32,
    version_header_id: i32,
    token: &str,
) -> Result<()> {
    // path code
    let mut files: Vec<(String, String)> = vec![];
    for path in paths.into_iter() {
        let origin_path = Path::new(&path);
        if !Path::exists(origin_path) {
            eprintln!("No such file {}", path);
            exit(1);
        }
        let file = fs::read_to_string(&path).unwrap();
        files.push((path, file));
    }
    for (path, file) in files {
        let mut i = 0;
        loop {
            let uploaded = upload_file(
                &package_source,
                registry_id,
                version_header_id,
                &file,
                &path,
                token,
            )
            .await;
            match uploaded {
                Ok(_) => break,
                Err(e) => {
                    i += 1;
                    if i >= 5 {
                        return Err(e);
                    }
                    continue;
                }
            }
        }
    }
    Ok(())
}

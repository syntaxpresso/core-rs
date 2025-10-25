use std::path::PathBuf;

use crate::{
    common::{
        ts_file::TSFile,
        utils::path_util::{find_all_files_by_ext, find_all_files_by_extension},
    },
    responses::get_all_files_response::GetAllFilesCommandResponse,
};

pub fn execute(cwd: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let ext = "java";
    let files = find_all_files_by_extension(&cwd, ext);
    let tsfiles: Vec<TSFile> = find_all_files_by_ext(&cwd, ext);

    let files_size = files.len();

    // Print tree for each TSFile
    for tsfile in &tsfiles {
        if let Some(tree) = &tsfile.tree {
            let root = tree.root_node();
            println!("Parse tree for file {:?}:", tsfile.file_path());
            println!("{}", root.to_sexp());
        } else {
            println!("No tree for file: {:?}", tsfile.file_path());
        }
    }

    let response = GetAllFilesCommandResponse {
        command: "get-all-files".to_string(),
        cwd: cwd.display().to_string(),
        files,
        files_count: files_size,
    };
    let json = serde_json::to_string_pretty(&response)?;
    Ok(json)
}

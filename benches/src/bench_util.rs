use glob::glob;
use std::{fs::File, io, path::PathBuf};

pub fn get_fixtures() -> Vec<(String, String)> {
    glob("./fixtures/*.vue")
        .expect("Failed to load fixtures")
        .filter_map(Result::ok)
        .map(open_vue_file)
        .filter_map(Result::ok)
        .collect()
}

fn open_vue_file(path: PathBuf) -> io::Result<(String, String)> {
    use std::io::Read;
    // TODO: use file_name after https://github.com/benchmark-action/github-action-benchmark/pull/80
    let name = path.file_stem().expect("Fixture should be file");
    let name = name.to_str().expect("Invalid fixture file name").to_owned();
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok((name, s))
}

use std::{env, path::PathBuf, process::Command};

fn main() {
    let app = env::args()
        .next()
        .expect("Runner-Error: No application to run.");

    let project_folder = PathBuf::from(file!());
    let f = project_folder.parent().unwrap().parent();

    println!("{f:?}")
}

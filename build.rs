use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/hello.c");

    if cfg!(target_os = "macos") {
        let _ = Command::new("git")
            .args([
                "clone",
                "https://github.com/arminbiere/kissat.git",
                "kissat_files",
            ])
            .output()
            .expect("failed to execute process");

        let _ = Command::new("./configure")
            .current_dir("./kissat_files")
            .output()
            .expect("failed to execute process");

        let _ = Command::new("make")
            .current_dir("./kissat_files")
            .args(["test"])
            .output()
            .expect("failed to execute process");

        let out = Command::new("mv")
            .args(["./kissat_files/build/kissat", "./kissat"])
            .output()
            .expect("failed to execute process");
    }
}

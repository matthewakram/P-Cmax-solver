use std::process::Command;

fn main() {
    if cfg!(target_os = "macos") {
        let out = Command::new("git")
            .args([
                "clone",
                "https://github.com/arminbiere/kissat.git",
                "kissat_files",
            ])
            .output()
            .expect("failed to execute process");

        let str = std::str::from_utf8(&out.stdout).unwrap();
        println!("{}", str);



        let out = Command::new("./configure")
            .current_dir("./kissat_files")
            .output()
            .expect("failed to execute process");

            let str = std::str::from_utf8(&out.stdout).unwrap();
            println!("{}", str);

        let out = Command::new("make")
            .current_dir("./kissat_files")
            .args(["test"])
            .output()
            .expect("failed to execute process");

            let str = std::str::from_utf8(&out.stdout).unwrap();
            println!("{}", str);

        let out = Command::new("mv")
            .args(["./kissat_files/build/kissat", "./kissat"])
            .output()
            .expect("failed to execute process");

            let str = std::str::from_utf8(&out.stdout).unwrap();
            println!("{}", str);

            assert!(false);
    }
}

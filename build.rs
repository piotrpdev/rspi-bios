use std::process::Command;

// https://stackoverflow.com/a/44407625/19020549

#[allow(clippy::unnecessary_wraps)]
fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    else {
        return Err("Failed to get current Git commit using command".into());
    };

    let Ok(git_hash) = String::from_utf8(output.stdout) else {
        return Err("Failed to convert Git output to UTF-8 string".into());
    };

    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:#?}");
        std::process::exit(1)
    }
}

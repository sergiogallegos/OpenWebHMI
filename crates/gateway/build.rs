use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=OPENWEBHMI_BUILD_COMMIT");
    track_git_head();
    let commit = std::env::var("OPENWEBHMI_BUILD_COMMIT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(git_commit)
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=OPENWEBHMI_GIT_COMMIT={commit}");
}

fn track_git_head() {
    const GIT_HEAD: &str = "../../.git/HEAD";
    println!("cargo:rerun-if-changed={GIT_HEAD}");
    if let Ok(head) = std::fs::read_to_string(GIT_HEAD)
        && let Some(reference) = head.trim().strip_prefix("ref: ")
    {
        println!("cargo:rerun-if-changed=../../.git/{reference}");
    }
}

fn git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

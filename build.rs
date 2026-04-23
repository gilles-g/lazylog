use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LL_GIT_COMMIT={commit}");

    let build_date = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LL_BUILD_DATE={build_date}");

    println!("cargo:rustc-env=LL_OS={}", std::env::consts::OS);
    println!("cargo:rustc-env=LL_ARCH={}", std::env::consts::ARCH);

    println!("cargo:rerun-if-changed=.git/HEAD");
}

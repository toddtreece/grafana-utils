use std::io::Write;
use std::process::Command;

pub fn build() {
    let cmd = Command::new("mage")
        .args(["-v"])
        .output()
        .expect("failed to build plugin");

    std::io::stdout().write_all(&cmd.stdout).unwrap();
    std::io::stderr().write_all(&cmd.stderr).unwrap();
}

pub fn certificate() -> Vec<u8> {
    let cmd = Command::new("mage")
        .args(["e2e:certificate"])
        .output()
        .expect("failed to get E2E certificate");

    cmd.stdout
}

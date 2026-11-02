//! Release invariants that the manifests cannot enforce on their own.

use std::process::Command;

#[test]
#[cfg_attr(miri, ignore = "Miri cannot run cargo")]
fn the_derive_dependency_is_pinned_to_this_release() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let requirement = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == "presence-rs")
        .and_then(|package| {
            package["dependencies"]
                .as_array()
                .unwrap()
                .iter()
                .find(|dependency| dependency["name"] == "presence-rs-derive")
        })
        .map(|dependency| dependency["req"].as_str().unwrap().to_owned());
    assert_eq!(
        requirement,
        Some(format!("={}", env!("CARGO_PKG_VERSION"))),
        "presence-rs-derive must be pinned to the workspace version"
    );
}

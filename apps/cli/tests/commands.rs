use std::process::Command;

#[test]
fn list_is_machine_readable_and_unknown_models_fail_with_json() {
    let list = Command::new(env!("CARGO_BIN_EXE_rsphysics"))
        .args(["models", "list"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let models: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(models.as_array().unwrap().len(), 22);
    for id in ["ecosystem.fvcb", "ecosystem.fvcb-big-leaf"] {
        assert!(
            models
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["metadata"]["id"] == id)
        );
    }
    assert!(
        models
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["metadata"]["id"] == "surface.ross-li")
    );
    assert!(
        models
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["metadata"]["id"] == "vegetation.prospect-d")
    );
    assert!(
        models
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["metadata"]["id"] == "vegetation.prosail-d")
    );
    let unknown = Command::new(env!("CARGO_BIN_EXE_rsphysics"))
        .args(["models", "describe", "test.unavailable"])
        .output()
        .unwrap();
    assert!(!unknown.status.success());
    assert!(unknown.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&unknown.stderr).unwrap();
    assert_eq!(error["kind"], "unknown_model");
}

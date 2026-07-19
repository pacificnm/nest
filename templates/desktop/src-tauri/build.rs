// Commands exposed by the inline `nest-desktop-template` Tauri plugin (see
// `main.rs`). Listing them here lets `tauri-build` autogenerate
// `allow-*`/`deny-*` ACL permissions and a `nest-desktop-template:default`
// set — without this, Tauri v2 denies every `plugin:nest-desktop-template|*`
// invoke with "plugin not found". Keep in sync with `main.rs`'s
// `generate_handler!` call.
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().plugin(
            "nest-desktop-template",
            tauri_build::InlinedPlugin::new()
                .commands(&["run_cli"])
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to run tauri-build");
}

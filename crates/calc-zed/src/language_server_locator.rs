use calc_lsp::LANGUAGE_SERVER_BINARY;
use zed_extension_api as zed;

pub(crate) fn command(worktree: &zed::Worktree) -> zed::Command {
    let bundled_binary = format!("bin/{LANGUAGE_SERVER_BINARY}");
    if executable_exists(&bundled_binary) {
        return zed::Command::new(format!("./{bundled_binary}"));
    }

    let workspace_binary = format!(
        "{}/target/debug/{}",
        worktree.root_path(),
        LANGUAGE_SERVER_BINARY
    );
    if executable_exists(&workspace_binary) {
        return zed::Command::new(workspace_binary);
    }

    if let Some(path) = worktree.which(LANGUAGE_SERVER_BINARY) {
        return zed::Command::new(path);
    }

    zed::Command::new(workspace_binary)
}

fn executable_exists(path: &str) -> bool {
    let mut command = zed::Command::new("test").args(["-x", path]);
    command
        .output()
        .is_ok_and(|output| output.status == Some(0))
}

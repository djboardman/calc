use calc_lsp::LANGUAGE_SERVER_BINARY;
use zed_extension_api as zed;

pub(crate) fn command(worktree: &zed::Worktree) -> zed::Result<zed::Command> {
    let bundled_binary = format!("bin/{LANGUAGE_SERVER_BINARY}");
    if executable_exists(&bundled_binary) {
        return Ok(zed::Command::new(format!("./{bundled_binary}")));
    }

    if let Some(path) = worktree.which(LANGUAGE_SERVER_BINARY) {
        return Ok(zed::Command::new(path));
    }

    Err(format!(
        "{LANGUAGE_SERVER_BINARY} must be built and installed on PATH or bundled under bin"
    ))
}

fn executable_exists(path: &str) -> bool {
    let mut command = zed::Command::new("test").args(["-x", path]);
    command
        .output()
        .is_ok_and(|output| output.status == Some(0))
}

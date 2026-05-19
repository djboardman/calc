use calc_lsp::LANGUAGE_SERVER_ID;
use zed_extension_api as zed;

use crate::{configuration::Configuration, language_server_locator};

pub struct CalcZed {
    _configuration: Configuration,
}

impl zed::Extension for CalcZed {
    fn new() -> Self {
        Self {
            _configuration: Configuration,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != LANGUAGE_SERVER_ID {
            return Err(format!(
                "unsupported language server id `{language_server_id}`"
            ));
        }

        language_server_locator::command(worktree)
    }
}

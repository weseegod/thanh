//! `/clear` -- wipe the on-screen transcript without ending the session.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Clear the TUI transcript while keeping session context intact.
pub struct ClearCommand;

impl SlashCommand for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn description(&self) -> &str {
        "Clear the transcript from the screen (keeps session context)"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn usage(&self) -> &str {
        "/clear"
    }

    fn run(&self, ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        if ctx.session_id.is_none() {
            return CommandResult::Error("No active session".to_string());
        }

        CommandResult::Action(Action::ClearDisplay)
    }
}

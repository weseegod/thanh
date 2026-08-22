# Enterprise configuration pinning

Hand-maintained mirror of the `[features]` registry
(`xai-grok-config-types/src/registry.rs`). Remote settings may pin any row;
a pinned row wins over `config.toml`, env vars, and defaults.

| Feature | Config key | Config path | Environment variable | Default | Pin via remote settings |
|---------|-----------|-------------|----------------------|---------|------------------------|
| The `ask_user_question` tool. | `ask_user_question` | `features.ask_user_question` | `GROK_ASK_USER_QUESTION` | enabled | yes |
| Continue the conversation as soon as a background task or subagent finishes. | `auto_wake` | `features.auto_wake` | `GROK_AUTO_WAKE` | enabled | yes |
| Server-side execution of `web_search` and `x_search`. | `backend_tools` | `features.backend_tools` | `GROK_BACKEND_SEARCH` | enabled | yes |
| Ctrl+C before a turn's first activity restores the prompt. | `cancel_rewind` | `features.cancel_rewind` | `GROK_CANCEL_REWIND` | enabled | yes |
| Summarize the verbatim conversation rather than a shortened copy of it. | `compaction_verbatim_input` | `features.compaction_verbatim_input` | `GROK_COMPACTION_VERBATIM_INPUT` | enabled | yes |
| Heuristic feedback popups and the `/feedback` command. | `feedback` | `features.feedback` | `GROK_FEEDBACK_ENABLED` | enabled | yes |
| Language-server-backed navigation tools. | `lsp_tools` | `features.lsp_tools` | `GROK_LSP_TOOLS` | disabled | yes |
| `/recap` and the automatic return-from-away recap. | `session_recap` | `features.session_recap` | `GROK_SESSION_RECAP` | enabled | yes |
| The SQLite session-search index. | `session_search` | `features.session_search` | `GROK_SESSION_SEARCH` | enabled | yes |
| Save a finished subagent's working copy into the repo as a git ref, restored on resume. | `subagent_worktree_snapshot` | `features.subagent_worktree_snapshot` | `GROK_SUBAGENT_WORKTREE_SNAPSHOT` | disabled | yes |
| The per-turn summary on the agent dashboard. | `turn_summary` | `features.turn_summary` | `GROK_TURN_SUMMARY` | enabled | yes |
| Summarize the earlier part of a long conversation in the background, before compaction. | `two_pass_compaction` | `features.two_pass_compaction` | `GROK_TWO_PASS_COMPACTION` | disabled | yes |
| Voice dictation (speech to text). | `voice_mode` | `features.voice_mode` | `GROK_VOICE_MODE` | enabled | yes |
| The `web_fetch` tool. | `web_fetch` | `features.web_fetch` | `GROK_WEB_FETCH` | disabled | yes |
| The `write_file` tool. | `write_file` | `features.write_file` | `GROK_WRITE_FILE` | enabled | yes |

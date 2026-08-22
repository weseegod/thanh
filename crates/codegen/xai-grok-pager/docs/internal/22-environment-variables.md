# Environment variables

Hand-maintained mirror of the `[features]` registry
(`xai-grok-config-types/src/registry.rs`). Every feature flag has an
environment override read at process start.

| Environment variable | Config key | Effect | Default |
|----------------------|-----------|--------|---------|
| `GROK_ASK_USER_QUESTION` | `ask_user_question` | Enables/disables The `ask_user_question` tool. when set to `true`/`false` | enabled |
| `GROK_AUTO_WAKE` | `auto_wake` | Enables/disables Continue the conversation as soon as a background task or subagent finishes. when set to `true`/`false` | enabled |
| `GROK_BACKEND_SEARCH` | `backend_tools` | Enables/disables Server-side execution of `web_search` and `x_search`. when set to `true`/`false` | enabled |
| `GROK_CANCEL_REWIND` | `cancel_rewind` | Enables/disables Ctrl+C before a turn's first activity restores the prompt. when set to `true`/`false` | enabled |
| `GROK_COMPACTION_VERBATIM_INPUT` | `compaction_verbatim_input` | Enables/disables Summarize the verbatim conversation rather than a shortened copy of it. when set to `true`/`false` | enabled |
| `GROK_FEEDBACK_ENABLED` | `feedback` | Enables/disables Heuristic feedback popups and the `/feedback` command. when set to `true`/`false` | enabled |
| `GROK_LSP_TOOLS` | `lsp_tools` | Enables/disables Language-server-backed navigation tools. when set to `true`/`false` | disabled |
| `GROK_SESSION_RECAP` | `session_recap` | Enables/disables `/recap` and the automatic return-from-away recap. when set to `true`/`false` | enabled |
| `GROK_SESSION_SEARCH` | `session_search` | Enables/disables The SQLite session-search index. when set to `true`/`false` | enabled |
| `GROK_SUBAGENT_WORKTREE_SNAPSHOT` | `subagent_worktree_snapshot` | Enables/disables Save a finished subagent's working copy into the repo as a git ref, restored on resume. when set to `true`/`false` | disabled |
| `GROK_TURN_SUMMARY` | `turn_summary` | Enables/disables The per-turn summary on the agent dashboard. when set to `true`/`false` | enabled |
| `GROK_TWO_PASS_COMPACTION` | `two_pass_compaction` | Enables/disables Summarize the earlier part of a long conversation in the background, before compaction. when set to `true`/`false` | disabled |
| `GROK_VOICE_MODE` | `voice_mode` | Enables/disables Voice dictation (speech to text). when set to `true`/`false` | enabled |
| `GROK_WEB_FETCH` | `web_fetch` | Enables/disables The `web_fetch` tool. when set to `true`/`false` | disabled |
| `GROK_WRITE_FILE` | `write_file` | Enables/disables The `write_file` tool. when set to `true`/`false` | enabled |

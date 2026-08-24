//! Plan-mode subagent gate through the real `prepare_tool_call` path plus
//! direct gate-unit checks: while plan mode is active, spawning a
//! write-capable subagent (general-purpose, codex, ...) is rejected with a
//! clear model-facing message, while read-only explore spawns stay allowed
//! (including parallel ones). Unknown/unresolvable types fail closed.
use super::support::*;
use super::*;
/// Build an actor whose toolset parses the task/spawn tool (plus the
/// background-task helpers the task tool requires) and the plan tools, so
/// `prepare_tool_call` can parse a genuine `spawn_subagent` call and the
/// rejection message's `${{ tools.by_kind.task }}` resolves.
async fn build_subagent_gate_actor() -> SessionActor {
    use xai_grok_tools::implementations::grok_build::enter_plan_mode::EnterPlanModeTool;
    use xai_grok_tools::implementations::grok_build::exit_plan_mode::ExitPlanModeTool;
    use xai_grok_tools::implementations::grok_build::kill_task::KillTaskTool;
    use xai_grok_tools::implementations::grok_build::task::TaskTool;
    use xai_grok_tools::implementations::grok_build::task_output::TaskOutputTool;
    use xai_grok_tools::implementations::grok_build::task_output::wait_tasks::WaitTasksTool;
    use xai_grok_tools::registry::types::ToolConfig;
    let (gateway_tx, mut gateway_rx) =
        tokio::sync::mpsc::unbounded_channel::<xai_acp_lib::AcpClientMessage>();
    let (persistence_tx, _persistence_rx) =
        tokio::sync::mpsc::unbounded_channel::<PersistenceMsg>();
    let actor = create_test_actor(0, 256_000, 85, gateway_tx, persistence_tx).await;
    *actor.agent.borrow_mut() = test_agent_with_tools(vec![
        ToolConfig::for_tool::<TaskTool>(),
        ToolConfig::for_tool::<TaskOutputTool>(),
        ToolConfig::for_tool::<KillTaskTool>(),
        ToolConfig::for_tool::<WaitTasksTool>(),
        ToolConfig::for_tool::<EnterPlanModeTool>(),
        ToolConfig::for_tool::<ExitPlanModeTool>(),
    ])
    .await;
    tokio::task::spawn_local(async move {
        while let Some(msg) = gateway_rx.recv().await {
            if let xai_acp_lib::AcpClientMessage::SessionNotification(args) = msg {
                let _ = args.response_tx.send(Ok(()));
            }
        }
    });
    actor
}
/// Flip the fixture's tracker to Active (plan file: `/tmp/test-session/plan.md`).
fn activate_plan_mode(actor: &SessionActor) {
    let mut tracker = actor.plan_mode.lock();
    assert!(tracker.enter_pending());
    assert!(tracker.activate());
}
fn spawn_call(id: &str, subagent_type: &str) -> ToolCallResponse {
    ToolCallResponse {
        id: id.to_string(),
        kind: "function".to_string(),
        function: crate::sampling::types::ToolCallFunction::new(
            "task",
            format!(
                r#"{{"prompt":"probe","description":"probe","subagent_type":"{subagent_type}"}}"#
            ),
        ),
    }
}
async fn prepare(
    actor: &SessionActor,
    call: ToolCallResponse,
) -> Result<PreparedToolCall, ToolLoop> {
    let mut deferred = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        actor.prepare_tool_call(call, &mut deferred),
    )
    .await
    .expect("prepare_tool_call must not hang (a hang means a permission prompt was issued)")
    .expect("prepare_tool_call must not error")
}
/// Last tool_result pushed for `call_id`, or panic.
async fn tool_result_text(actor: &SessionActor, call_id: &str) -> String {
    let conv = actor.chat_state_handle.get_conversation().await;
    conv.iter()
        .rev()
        .find_map(|item| match item {
            xai_grok_sampling_types::ConversationItem::ToolResult(tr)
                if tr.tool_call_id == call_id =>
            {
                Some(tr.content.to_string())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no tool_result for {call_id} in {conv:?}"))
}
/// The headline: plan mode Active + allow-all permissions (the always-approve
/// worst case) still rejects a write-capable subagent spawn, with a message
/// naming the plan file and redirecting to read-only explore spawns — and
/// WITHOUT steering to `exit_plan_mode` (mirror of the edit gate message).
#[tokio::test(flavor = "current_thread")]
async fn plan_mode_rejects_write_capable_subagent_spawn_despite_allow_all_permissions() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let actor = build_subagent_gate_actor().await;
            activate_plan_mode(&actor);
            let result = prepare(&actor, spawn_call("call_gp", "general-purpose")).await;
            assert!(
                matches!(result, Err(ToolLoop::Continue)),
                "gate must reject with Continue (tool not executed); got {result:?}"
            );
            let text = tool_result_text(&actor, "call_gp").await;
            assert!(
                text.contains(
                    "Rejected: spawning write-capable subagents is not allowed in plan mode"
                ),
                "rejection text: {text}"
            );
            assert!(
                text.contains("only editable file is the plan file"),
                "must name the plan-file rule: {text}"
            );
            assert!(
                text.contains("/tmp/test-session/plan.md"),
                "must name the plan file so the model knows the one editable path: {text}"
            );
            assert!(
                text.contains("subagent_type=\"explore\""),
                "must redirect to read-only explore spawns: {text}"
            );
            assert!(
                text.contains("task tool"),
                "task tool name hint should resolve from the registry: {text}"
            );
            assert!(
                !text.contains("exit_plan_mode"),
                "rejection should stay short (no exit-tool steering): {text}"
            );
            assert!(
                !text.contains("${{"),
                "unresolved template placeholder: {text}"
            );
        })
        .await;
}
/// A second write-capable builtin (codex) is rejected the same way.
#[tokio::test(flavor = "current_thread")]
async fn plan_mode_rejects_codex_subagent_spawn() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let actor = build_subagent_gate_actor().await;
            activate_plan_mode(&actor);
            let result = prepare(&actor, spawn_call("call_codex", "codex")).await;
            assert!(
                matches!(result, Err(ToolLoop::Continue)),
                "codex spawn must be rejected in plan mode; got {result:?}"
            );
            let text = tool_result_text(&actor, "call_codex").await;
            assert!(
                text.contains(
                    "Rejected: spawning write-capable subagents is not allowed in plan mode"
                ),
                "rejection text: {text}"
            );
        })
        .await;
}
/// An unknown / unresolvable type fails CLOSED: the spawn would fail
/// `Unknown` anyway, and rejecting early with the clear plan-mode message is
/// the design's chosen behavior.
#[tokio::test(flavor = "current_thread")]
async fn plan_mode_rejects_unknown_subagent_type_fail_closed() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let actor = build_subagent_gate_actor().await;
            activate_plan_mode(&actor);
            let result = prepare(&actor, spawn_call("call_unknown", "totally-invented")).await;
            assert!(
                matches!(result, Err(ToolLoop::Continue)),
                "unknown type must fail closed in plan mode; got {result:?}"
            );
            let text = tool_result_text(&actor, "call_unknown").await;
            assert!(
                text.contains(
                    "Rejected: spawning write-capable subagents is not allowed in plan mode"
                ),
                "rejection text: {text}"
            );
        })
        .await;
}
/// Direct gate checks (no actor): inactive plan mode allows every spawn,
/// read-only explore stays allowed while active, and the gate is a no-op for
/// non-task tools.
#[tokio::test(flavor = "current_thread")]
async fn direct_gate_inactive_allows_and_read_only_explore_allowed() {
    use crate::session::plan_mode::PlanModeTracker;
    use xai_grok_tools::types::ToolInput;
    use xai_tool_types::TaskToolInput;
    let inactive = PlanModeTracker::new(std::path::PathBuf::from("/tmp/test-session"));
    let cwd = std::path::Path::new("/tmp");
    let explore = ToolInput::Task(TaskToolInput {
        prompt: "p".into(),
        description: "d".into(),
        subagent_type: "explore".into(),
        run_in_background: true,
        capability_mode: None,
        isolation: None,
        resume_from: None,
        cwd: None,
        model: None,
        task_id: None,
    });
    assert_eq!(
        super::tool_calls::plan_mode_subagent_gate(&inactive, &explore, cwd, None),
        super::tool_calls::PlanEditGate::Allow,
        "inactive plan mode must not gate subagent spawns"
    );
    let mut active = PlanModeTracker::new(std::path::PathBuf::from("/tmp/test-session"));
    assert!(active.enter_pending());
    assert!(active.activate());
    assert_eq!(
        super::tool_calls::plan_mode_subagent_gate(&active, &explore, cwd, None),
        super::tool_calls::PlanEditGate::Allow,
        "read-only explore spawns must stay allowed in plan mode (parallel exploration)"
    );
    let gp = ToolInput::Task(TaskToolInput {
        prompt: "p".into(),
        description: "d".into(),
        subagent_type: "general-purpose".into(),
        run_in_background: true,
        capability_mode: None,
        isolation: None,
        resume_from: None,
        cwd: None,
        model: None,
        task_id: None,
    });
    assert_eq!(
        super::tool_calls::plan_mode_subagent_gate(&active, &gp, cwd, None),
        super::tool_calls::PlanEditGate::RejectWriteCapableSubagent,
        "write-capable general-purpose must be rejected in plan mode"
    );
    let unknown = ToolInput::Task(TaskToolInput {
        prompt: "p".into(),
        description: "d".into(),
        subagent_type: "totally-invented".into(),
        run_in_background: true,
        capability_mode: None,
        isolation: None,
        resume_from: None,
        cwd: None,
        model: None,
        task_id: None,
    });
    assert_eq!(
        super::tool_calls::plan_mode_subagent_gate(&active, &unknown, cwd, None),
        super::tool_calls::PlanEditGate::RejectWriteCapableSubagent,
        "unknown type must fail closed in plan mode"
    );
    let bash = ToolInput::Bash(xai_grok_tools::implementations::BashToolInput {
        command: "echo hi > /tmp/f".into(),
        timeout: None,
        description: "write via bash".into(),
        is_background: false,
    });
    assert_eq!(
        super::tool_calls::plan_mode_subagent_gate(&active, &bash, cwd, None),
        super::tool_calls::PlanEditGate::Allow,
        "non-task tools are not subagent-gated (edit gate / permission path handle them)"
    );
}
/// Control: with plan mode inactive the same general-purpose spawn prepares
/// cleanly — the gate is plan-scoped, not a general spawn block.
#[tokio::test(flavor = "current_thread")]
async fn inactive_plan_mode_does_not_gate_spawns_via_actor_prepare() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let actor = build_subagent_gate_actor().await;
            // No activate_plan_mode: tracker stays Inactive, so the gate's
            // first check (is_active) returns Allow immediately.
            let result = prepare(&actor, spawn_call("call_no_plan", "general-purpose")).await;
            assert!(
                result.is_ok(),
                "spawn outside plan mode must prepare; got {:?}",
                result.err()
            );
        })
        .await;
}

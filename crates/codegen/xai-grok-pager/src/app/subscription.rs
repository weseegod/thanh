//! Server-driven gate imposition/lift.
//!
//! The fork removes the consumer free→paid paywall (BYOK sessions have no
//! grok.com billing), but keeps the gate chokepoint: a gate can still arrive
//! from remote settings (`grok_build_settings.gate_message`), and it must
//! render and lift through one place. `impose_gate` shows directly — the
//! consumer "defer while a live subscription check verifies" dance is gone.

use super::actions::Effect;
use super::app_view::AppView;

impl AppView {
    /// Chokepoint for showing a gate. Already gated → update the copy.
    /// Otherwise show directly (no consumer deferral — see module docs).
    #[must_use]
    pub fn impose_gate(&mut self, gate: xai_grok_shell::auth::GateInfo) -> Vec<Effect> {
        if self.gate.is_some() {
            self.gate = Some(gate);
            return vec![];
        }
        crate::unified_log::info(
            "subscription.gate.imposed",
            None,
            Some(serde_json::json!({ "deferred": false })),
        );
        self.gate = Some(gate);
        vec![]
    }

    /// Chokepoint for a settings-confirmed gate lift. Clears the visible
    /// gate and runs the lift bookkeeping (re-focus the welcome prompt).
    #[must_use]
    pub fn lift_gate(&mut self) -> Vec<Effect> {
        let was_blocked = self.gate.is_some();
        self.gate = None;
        if !was_blocked {
            return vec![];
        }
        self.welcome_prompt_focused = true;
        crate::unified_log::info(
            "subscription.gate.lifted",
            None,
            Some(serde_json::json!({ "tier": self.subscription_tier })),
        );
        vec![]
    }
}

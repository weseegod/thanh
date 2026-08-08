//! Tabbed usage / session-info modal, opened by `/usage`, `/session-info`,
//! `/context`, and the context-bar click. Minimal mode keeps the scrollback
//! blocks instead; this modal is never armed there.
//!
//! The modal opens with loading placeholders; the task-result handlers fill
//! the slots in as the fetches land.

use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::scrollback::blocks::ContextInfoBlock;
use crate::theme::Theme;
use crate::views::modal_window::{
    self as mw, ModalSizing, ModalWindowConfig, ModalWindowState, Shortcut,
};

/// Footer shortcut ID for "copy session ID".
pub const COPY_SESSION_ID_SHORTCUT: usize = 1;

/// The three tabs, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageInfoTab {
    ContextUsage,
    Usage,
    SessionInfo,
}

impl UsageInfoTab {
    pub const ALL: [UsageInfoTab; 3] = [
        UsageInfoTab::ContextUsage,
        UsageInfoTab::Usage,
        UsageInfoTab::SessionInfo,
    ];

    pub fn label(self) -> &'static str {
        match self {
            UsageInfoTab::ContextUsage => "Context usage",
            UsageInfoTab::Usage => "Usage",
            UsageInfoTab::SessionInfo => "Session info",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Self {
        *Self::ALL.get(i).unwrap_or(&Self::ALL[0])
    }
}

/// Account/session facts captured when the modal opens.
pub struct UsageInfoContext {
    /// Session ID for the copy shortcut (`None` before the session starts).
    pub session_id: Option<String>,
}

/// Modal state. Billing figures are NOT stored here — the render reads the
/// agent's cached `credit_balance` mirror, so a silent billing refresh
/// updates the open modal for free.
pub struct UsageInfoModalState {
    pub window: ModalWindowState,
    pub active_tab: UsageInfoTab,
    pub scroll: u16,
    pub ctx: UsageInfoContext,
    pub context: Option<ContextInfoBlock>,
    pub context_error: Option<String>,
    /// Pre-formatted `/session-info` text (built by `format_session_info`).
    pub session_text: Option<String>,
    pub session_error: Option<String>,
    /// Pre-formatted session token/cost summary (`session_usage_block_text`).
    pub session_usage_text: Option<String>,
    /// Fetch generation stamped at open; results from an earlier open (same
    /// session, modal reopened) are dropped instead of overwriting.
    pub fetch_nonce: u64,
    /// Hit rect of the visible "Session ID" row (click-to-copy), refreshed
    /// every render.
    pub session_id_rect: Option<Rect>,
}

impl UsageInfoModalState {
    pub fn new(tab: UsageInfoTab, ctx: UsageInfoContext) -> Self {
        Self {
            window: ModalWindowState::with_tabs(UsageInfoTab::ALL.len()),
            active_tab: tab,
            scroll: 0,
            ctx,
            context: None,
            context_error: None,
            session_text: None,
            session_error: None,
            session_usage_text: None,
            fetch_nonce: 0,
            session_id_rect: None,
        }
    }

    pub fn set_tab(&mut self, tab: UsageInfoTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.scroll = 0;
        }
    }

    fn step_tab(&mut self, forward: bool) {
        let n = UsageInfoTab::ALL.len();
        let i = self.active_tab.index();
        let next = if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        };
        self.set_tab(UsageInfoTab::from_index(next));
    }
}

/// Outcome of a content key/mouse event. Chrome events (Esc, `[✗]`, tab
/// clicks, footer clicks) are handled by the caller via `modal_window`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageModalOutcome {
    /// Copy the session ID to the clipboard (caller owns clipboard + toast).
    CopySessionId,
    Changed,
    Unchanged,
}

pub fn handle_usage_modal_key(
    state: &mut UsageInfoModalState,
    key: &KeyEvent,
) -> UsageModalOutcome {
    use crossterm::event::KeyModifiers;
    // BackTab / `G` legitimately carry SHIFT; reject only real chords.
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return UsageModalOutcome::Unchanged;
    }
    match key.code {
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
            state.step_tab(true);
            UsageModalOutcome::Changed
        }
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
            state.step_tab(false);
            UsageModalOutcome::Changed
        }
        KeyCode::Char(c @ '1'..='3') => {
            state.set_tab(UsageInfoTab::from_index(c as usize - '1' as usize));
            UsageModalOutcome::Changed
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll = state.scroll.saturating_sub(1);
            UsageModalOutcome::Changed
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll = state.scroll.saturating_add(1);
            UsageModalOutcome::Changed
        }
        KeyCode::PageUp => {
            state.scroll = state.scroll.saturating_sub(10);
            UsageModalOutcome::Changed
        }
        KeyCode::PageDown => {
            state.scroll = state.scroll.saturating_add(10);
            UsageModalOutcome::Changed
        }
        KeyCode::Home => {
            state.scroll = 0;
            UsageModalOutcome::Changed
        }
        // Scroll offsets are clamped to the content height at render time.
        KeyCode::End | KeyCode::Char('G') => {
            state.scroll = u16::MAX;
            UsageModalOutcome::Changed
        }
        KeyCode::Char('c') if state.ctx.session_id.is_some() => UsageModalOutcome::CopySessionId,
        _ => UsageModalOutcome::Unchanged,
    }
}

pub fn handle_usage_modal_mouse(
    state: &mut UsageInfoModalState,
    kind: MouseEventKind,
    column: u16,
    row: u16,
) -> UsageModalOutcome {
    match kind {
        MouseEventKind::ScrollUp => {
            state.scroll = state.scroll.saturating_sub(3);
            UsageModalOutcome::Changed
        }
        MouseEventKind::ScrollDown => {
            state.scroll = state.scroll.saturating_add(3);
            UsageModalOutcome::Changed
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left)
            if state.session_id_rect.is_some_and(|r| {
                column >= r.x && column < r.x + r.width && row >= r.y && row < r.y + r.height
            }) =>
        {
            UsageModalOutcome::CopySessionId
        }
        _ => UsageModalOutcome::Unchanged,
    }
}

pub fn render_usage_modal(
    buf: &mut Buffer,
    area: Rect,
    state: &mut UsageInfoModalState,
    compact: bool,
    theme: &Theme,
) {
    let labels: Vec<&str> = UsageInfoTab::ALL.iter().map(|t| t.label()).collect();
    state.window.active_tab = state.active_tab.index();

    let mut shortcuts: Vec<Shortcut> = vec![
        Shortcut {
            label: "Tab switch",
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: "\u{2191}/\u{2193} scroll",
            clickable: false,
            id: 0,
        },
    ];
    if state.ctx.session_id.is_some() {
        shortcuts.push(Shortcut {
            label: "c copy session ID",
            clickable: true,
            id: COPY_SESSION_ID_SHORTCUT,
        });
    }
    shortcuts.push(Shortcut {
        label: "Esc close",
        clickable: false,
        id: 0,
    });

    // v_pad / footer_lines pad the body top and bottom (shortcuts render
    // bottom-aligned, so the spare footer row reads as bottom padding).
    let sizing = ModalSizing {
        width_pct: 0.65,
        max_width: 100,
        min_width: 44,
        v_margin: 2,
        h_pad: 2,
        v_pad: 2,
        footer_lines: 3,
    }
    .with_compact(compact);
    // No border title — the tab bar is the header, as in the extensions modal.
    let config = ModalWindowConfig {
        title: "",
        tabs: Some(&labels),
        shortcuts: &shortcuts,
        sizing,
        fold_info: None,
    };

    // The chrome always fills `area` minus `v_margin`, so cap the height
    // ourselves: tall terminals would otherwise get a mostly-empty box.
    // 30 rows ≈ the widest tab's content (context grid + legend) + chrome.
    const MAX_MODAL_HEIGHT: u16 = 30;
    let outer = MAX_MODAL_HEIGHT + sizing.v_margin * 2;
    let area = if area.height > outer {
        Rect {
            x: area.x,
            y: area.y + (area.height - outer) / 2,
            width: area.width,
            height: outer,
        }
    } else {
        area
    };

    let Some(mca) = mw::render_modal_window(buf, area, &mut state.window, &config, theme) else {
        state.session_id_rect = None;
        return;
    };
    let content = mca.content;
    let tab = tab_lines(state, theme, content.width);
    // No wrapping: one row per logical line keeps the scroll clamp exact.
    let max_scroll = tab.lines.len().saturating_sub(content.height as usize);
    state.scroll = (state.scroll as usize).min(max_scroll) as u16;
    state.session_id_rect = tab.session_id_row.and_then(|idx| {
        let visible_row = idx.checked_sub(state.scroll as usize)?;
        (visible_row < content.height as usize).then(|| Rect {
            x: content.x,
            y: content.y + visible_row as u16,
            width: content.width,
            height: 1,
        })
    });
    let visible: Vec<Line> = tab
        .lines
        .into_iter()
        .skip(state.scroll as usize)
        .take(content.height as usize)
        .collect();
    Paragraph::new(visible).render(content, buf);
}

/// Rendered content of one tab.
struct TabContent {
    lines: Vec<Line<'static>>,
    /// Row index of the session-ID value (click-to-copy target).
    session_id_row: Option<usize>,
}

impl TabContent {
    fn from_lines(lines: Vec<Line<'static>>) -> Self {
        Self {
            lines,
            session_id_row: None,
        }
    }
}

fn tab_lines(
    state: &UsageInfoModalState,
    theme: &Theme,
    width: u16,
) -> TabContent {
    match state.active_tab {
        UsageInfoTab::ContextUsage => {
            TabContent::from_lines(context_tab_lines(state, theme, width))
        }
        UsageInfoTab::Usage => TabContent::from_lines(usage_lines(state, theme)),
        UsageInfoTab::SessionInfo => session_info_content(state, theme),
    }
}

fn header_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.text_primary)
        .add_modifier(Modifier::BOLD)
}

fn plain(theme: &Theme, s: impl Into<String>) -> Line<'static> {
    Line::styled(s.into(), Style::default().fg(theme.text_primary))
}

fn muted_line(theme: &Theme, s: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(s.into(), theme.muted()))
}

fn context_tab_lines(state: &UsageInfoModalState, theme: &Theme, width: u16) -> Vec<Line<'static>> {
    if let Some(error) = &state.context_error {
        return vec![muted_line(
            theme,
            format!("Couldn't load context usage: {error}"),
        )];
    }
    if let Some(block) = &state.context {
        return block.lines_for_width(theme, width);
    }
    if state.ctx.session_id.is_none() {
        return vec![muted_line(theme, "No active session.")];
    }
    vec![muted_line(theme, "Loading context usage\u{2026}")]
}

/// This session's token/cost totals (the fork strips the consumer billing
/// allowance — BYOK sessions have no grok.com credits to show).
fn usage_lines(state: &UsageInfoModalState, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(usage_text) = &state.session_usage_text {
        for (i, row) in usage_text.lines().enumerate() {
            if i == 0 {
                lines.push(Line::styled(row.to_string(), header_style(theme)));
            } else {
                lines.push(plain(theme, row));
            }
        }
    } else if state.ctx.session_id.is_some() {
        lines.push(muted_line(theme, "Loading session usage\u{2026}"));
    } else {
        lines.push(muted_line(theme, "No active session."));
    }
    lines
}

/// Model/runtime details rendered as one compact `Label: value` block; every
/// other field gets a spaced label-over-value group.
fn is_compact_session_field(label: &str) -> bool {
    matches!(
        label,
        "Model" | "Model Hash" | "API Backend" | "Sandbox" | "Turn" | "Context"
    )
}

fn session_info_content(state: &UsageInfoModalState, theme: &Theme) -> TabContent {
    if let Some(error) = &state.session_error {
        return TabContent::from_lines(vec![muted_line(
            theme,
            format!("Couldn't load session info: {error}"),
        )]);
    }
    let Some(text) = &state.session_text else {
        if state.ctx.session_id.is_none() {
            return TabContent::from_lines(vec![muted_line(theme, "No active session.")]);
        }
        return TabContent::from_lines(vec![muted_line(theme, "Loading session info\u{2026}")]);
    };

    let mut lines = vec![Line::styled("Session info", header_style(theme))];
    let mut session_id_row = None;
    let mut prev_compact = false;
    for row in text.lines() {
        let trimmed = row.trim_start();
        // The auth method (and its `grok login` upsell) is deliberately
        // not part of this surface.
        if trimmed.is_empty()
            || trimmed.starts_with("Auth method:")
            || trimmed.starts_with("Run `thanh login`")
            || trimmed.starts_with("Run `grok login`")
        {
            continue;
        }
        let Some((label, value)) = trimmed.split_once(": ").filter(|(l, _)| l.len() <= 24) else {
            lines.push(plain(theme, trimmed));
            continue;
        };
        let compact = is_compact_session_field(label);
        if !(compact && prev_compact) {
            lines.push(Line::default());
        }
        // The session-ID value is underlined: its row is click-to-copy.
        let mut value_style = Style::default().fg(theme.text_primary);
        if label == "Session ID" {
            value_style = value_style.add_modifier(Modifier::UNDERLINED);
        }
        if compact {
            lines.push(Line::from(vec![
                Span::styled(format!("{label}: "), theme.muted()),
                Span::styled(value.to_string(), value_style),
            ]));
        } else {
            let mut label_spans = vec![Span::styled(format!("{label}:"), theme.muted())];
            if label == "Session ID" {
                label_spans.push(Span::styled(
                    "   click to copy \u{b7} press c",
                    Style::default().fg(theme.gray_dim),
                ));
            }
            lines.push(Line::from(label_spans));
            lines.push(Line::from(Span::styled(value.to_string(), value_style)));
        }
        if label == "Session ID" {
            session_id_row = Some(lines.len() - 1);
        }
        prev_compact = compact;
    }
    TabContent {
        lines,
        session_id_row,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn state_with_session() -> UsageInfoModalState {
        UsageInfoModalState::new(
            UsageInfoTab::Usage,
            UsageInfoContext {
                session_id: Some("sid-123".to_string()),
            },
        )
    }

    #[test]
    fn tab_cycling_wraps_and_resets_scroll() {
        let mut state = state_with_session();
        state.scroll = 7;
        assert_eq!(
            handle_usage_modal_key(&mut state, &key(KeyCode::Tab)),
            UsageModalOutcome::Changed
        );
        assert_eq!(state.active_tab, UsageInfoTab::SessionInfo);
        assert_eq!(state.scroll, 0, "tab switch resets scroll");
        handle_usage_modal_key(&mut state, &key(KeyCode::Tab));
        assert_eq!(state.active_tab, UsageInfoTab::ContextUsage, "wraps");
        handle_usage_modal_key(&mut state, &key(KeyCode::BackTab));
        assert_eq!(state.active_tab, UsageInfoTab::SessionInfo, "wraps back");
    }

    #[test]
    fn copy_shortcut_requires_a_session_id() {
        let mut state = state_with_session();
        assert_eq!(
            handle_usage_modal_key(&mut state, &key(KeyCode::Char('c'))),
            UsageModalOutcome::CopySessionId
        );
        state.ctx.session_id = None;
        assert_eq!(
            handle_usage_modal_key(&mut state, &key(KeyCode::Char('c'))),
            UsageModalOutcome::Unchanged
        );
    }

    #[test]
    fn usage_tab_shows_session_totals_only() {
        let theme = Theme::current();
        let mut state = state_with_session();
        state.session_usage_text = Some("Session usage\nTokens: 1,234\nCost: $0.01".to_string());
        let lines = usage_lines(&state, &theme);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        assert_eq!(text[0], "Session usage");
        assert!(text.iter().any(|l| l.contains("Tokens: 1,234")));
        assert!(
            !text.iter().any(|l| l.contains("Weekly") || l.contains("limit")),
            "no billing allowance surface: {text:?}"
        );

        // Loading state for an active session.
        state.session_usage_text = None;
        let lines = usage_lines(&state, &theme);
        assert!(lines[0].to_string().contains("Loading session usage"));

        // No session at all.
        state.ctx.session_id = None;
        let lines = usage_lines(&state, &theme);
        assert!(lines[0].to_string().contains("No active session"));
    }

    #[test]
    fn render_smoke_shows_tabs_and_copy_shortcut() {
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let mut state = state_with_session();
        state.session_usage_text = Some("Session usage: no model calls yet.".to_string());
        let theme = Theme::current();
        render_usage_modal(&mut buf, area, &mut state, false, &theme);
        let text: String = (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
                    + "\n"
            })
            .collect();
        for needle in [
            "Context usage",
            "Usage",
            "Session info",
            "copy session ID",
            "Session usage: no model calls yet.",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
        }
        assert_eq!(state.window.tab_rects.len(), 3);
        assert!(state.window.close_button_rect.is_some());
    }

    #[test]
    fn session_id_row_is_click_to_copy() {
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let mut state = state_with_session();
        state.set_tab(UsageInfoTab::SessionInfo);
        state.session_text = Some("  Title: t\n  Session ID: sid-123".to_string());
        let theme = Theme::current();
        render_usage_modal(&mut buf, area, &mut state, false, &theme);
        let rect = state.session_id_rect.expect("session ID row visible");
        assert_eq!(
            handle_usage_modal_mouse(
                &mut state,
                MouseEventKind::Down(crossterm::event::MouseButton::Left),
                rect.x,
                rect.y,
            ),
            UsageModalOutcome::CopySessionId
        );
        // Clicks elsewhere in the content don't copy.
        assert_eq!(
            handle_usage_modal_mouse(
                &mut state,
                MouseEventKind::Down(crossterm::event::MouseButton::Left),
                rect.x,
                rect.y + 2,
            ),
            UsageModalOutcome::Unchanged
        );
        // Other tabs never expose the rect.
        state.set_tab(UsageInfoTab::ContextUsage);
        render_usage_modal(&mut buf, area, &mut state, false, &theme);
        assert!(state.session_id_rect.is_none());
    }

    #[test]
    fn popup_height_is_capped_on_tall_terminals() {
        let area = Rect::new(0, 0, 100, 60);
        let mut buf = Buffer::empty(area);
        let mut state = state_with_session();
        let theme = Theme::current();
        render_usage_modal(&mut buf, area, &mut state, false, &theme);
        let popup = state.window.popup_area.expect("popup rendered");
        assert_eq!(popup.height, 30);
        // Still vertically centered.
        assert_eq!(popup.y, (60 - 30) / 2);
    }

    #[test]
    fn session_info_tab_spaces_groups_and_compacts_model_block() {
        let mut state = state_with_session();
        state.session_text = Some(
            "  Title: t\n  Auth method: OAuth\n  Run `grok login` to switch.\n  \
             Session ID: sid-123\n  Working directory: /tmp\n  Model: Grok\n  Context: 1 / 2"
                .to_string(),
        );
        let theme = Theme::current();
        let tab = session_info_content(&state, &theme);
        let text: Vec<String> = tab.lines.iter().map(|l| l.to_string()).collect();
        assert_eq!(
            text,
            [
                "Session info",
                "",
                "Title:",
                "t",
                "",
                "Session ID:   click to copy \u{b7} press c",
                "sid-123",
                "",
                "Working directory:",
                "/tmp",
                "",
                "Model: Grok",
                "Context: 1 / 2",
            ]
        );
        assert_eq!(tab.session_id_row, Some(6), "value row is the copy target");
    }
}

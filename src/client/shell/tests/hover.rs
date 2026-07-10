use super::*;

fn motion(state: &mut ClientShellState, rect: Rect) -> ClientShellInput {
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: rect.x,
        row: rect.y,
        modifiers: KeyModifiers::empty(),
    })])
}

fn hover_state() -> ClientShellState {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    let mut snapshot = snapshot();
    let mut tab = snapshot.tabs[0].clone();
    tab.tab_id = "tab_2".into();
    tab.focused = false;
    snapshot.tabs.push(tab);
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    state.compose(106, 20).expect("composed frame");
    state
}

#[test]
fn hovering_chrome_tracks_stable_targets() {
    let mut state = hover_state();
    let tab = state.hits.tabs[1].0;
    motion(&mut state, tab);
    assert_eq!(
        state.hover,
        Some(HoverTarget::Tab {
            tab_id: "tab_2".into()
        })
    );

    let workspace = state.hits.workspaces[0].rect;
    motion(&mut state, workspace);
    assert_eq!(
        state.hover,
        Some(HoverTarget::Workspace {
            endpoint_id: ClientEndpointId::Local,
            workspace_id: "ws_1".into(),
        })
    );

    let new_tab = state.hits.new_tab;
    assert!(!new_tab.is_empty());
    motion(&mut state, new_tab);
    assert_eq!(
        state.hover,
        Some(HoverTarget::Button {
            kind: HoverButtonKind::NewTab
        })
    );
}

#[test]
fn hovering_pane_requests_endpoint_focus_in_terminal_and_resize_modes() {
    for mode in [ClientShellMode::Terminal, ClientShellMode::Resize] {
        let mut state = hover_state();
        state.mode = mode;
        let pane = state.hits.panes[0].inner_rect;
        assert!(
            motion(&mut state, pane).actions.is_empty(),
            "already focused"
        );
        state.hits.panes[0].pane_id = "pane_2".into();
        let outcome = motion(&mut state, pane);
        assert!(
            matches!(&outcome.actions[..], [ClientShellAction::Endpoint { request, .. }]
            if matches!(&request.method, crate::api::schema::Method::PaneFocus(target) if target.pane_id == "pane_2"))
        );
    }
}

#[test]
fn hovering_does_not_focus_during_selection_or_copy_mode() {
    let mut state = hover_state();
    let pane = state.hits.panes[0].inner_rect;
    state.hits.panes[0].pane_id = "pane_2".into();
    state.mode = ClientShellMode::Copy;
    assert!(motion(&mut state, pane).actions.is_empty());
    state.mode = ClientShellMode::Terminal;
    state.selection = Some(crate::selection::Selection::anchor(
        "pane_1".into(),
        0,
        0,
        None,
    ));
    assert!(motion(&mut state, pane).actions.is_empty());
    assert!(state.hover.is_none());
}

#[test]
fn hovering_tabs_highlights_inactive_tabs_and_preserves_focused_style() {
    let mut state = hover_state();
    for (index, expected) in [
        (1, state.config.palette.surface1),
        (0, state.config.palette.accent),
    ] {
        let rect = state.hits.tabs[index].0;
        motion(&mut state, rect);
        let buffer = state.compose(106, 20).unwrap().to_ratatui_buffer().unwrap();
        assert_eq!(buffer[(rect.x, rect.y)].bg, expected);
    }
    let rect = state.hits.new_tab;
    motion(&mut state, rect);
    let buffer = state.compose(106, 20).unwrap().to_ratatui_buffer().unwrap();
    assert_eq!(buffer[(rect.x, rect.y)].bg, state.config.palette.surface1);
}

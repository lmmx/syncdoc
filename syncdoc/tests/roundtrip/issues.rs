#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_app_state() {
    let temp = setup_test_project(&[ModuleConfig::new("app_state")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{app_state.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"{app_state.md,app_state/{AppState.md,AppState/{build_tree.md,cancel_move.md,command_buffer.md,cumulative_offset.md,current_node_index.md,current_view.md,editor_state.md,enter_detail_view.md,exit_detail_view.md,file_mode.md,file_offsets.md,files.md,find_next_node.md,find_prev_node.md,generate_edit_plan.md,get_current_section.md,get_current_section_index.md,get_indent.md,get_max_line_width.md,load_docs.md,mark_moved.md,message.md,move_section_down.md,move_section_in.md,move_section_out.md,move_section_to_bottom.md,move_section_to_top.md,move_section_up.md,move_state.md,moving_section_index.md,navigate_to_first.md,navigate_to_first_at_level.md,navigate_to_first_child.md,navigate_to_last.md,navigate_to_last_at_level.md,navigate_to_next_descendant.md,navigate_to_next_sibling.md,navigate_to_parent.md,navigate_to_prev_sibling.md,new.md,rebuild_file_offsets.md,rebuild_tree.md,rewrite_file_sections.md,save_current.md,save_section_reorder.md,sections.md,start_move.md,tree_nodes.md,wrap_width.md},FileMode.md,FileMode/{Multi.md,Single.md},MoveState.md,MoveState/{Moved.md,None.md,Selected.md},View.md,View/{Command.md,Detail.md,List.md}},lib.md}");

    result.snapshot_source_files("app_state");
    result.snapshot_docs_files("app_state");

    assert_snapshot!(
        "app_state_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (whitespace changes)"
        }
    );

    // Extra diagnostic: line count difference
    if let Some(diff) = result.get_file_diff("app_state.rs") {
        let original_lines = diff.original.lines().count();
        let restored_lines = diff.restored.lines().count();
        assert_snapshot!(
            "app_state_line_diff",
            format!(
                "Original: {} lines\nRestored: {} lines\nDiff: {}",
                original_lines,
                restored_lines,
                original_lines as i32 - restored_lines as i32
            )
        );
    }
}

#[test]
fn roundtrip_highlight() {
    let temp = setup_test_project(&[ModuleConfig::new("highlight")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{highlight,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{highlight.md,highlight/{SYNTAX_SET.md,THEME_SET.md,highlight_line_with_extension.md,highlight_source_lines.md},lib.md}");

    result.snapshot_source_files("highlight");
    result.snapshot_docs_files("highlight");

    assert_snapshot!(
        "highlight_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (omnidoc added)"
        }
    );

    // Extra diagnostic: omnidoc presence
    if let Some(diff) = result.get_file_diff("highlight.rs") {
        let original_omnidoc = diff.original.matches("#[syncdoc::omnidoc]").count();
        let restored_omnidoc = diff.restored.matches("#[syncdoc::omnidoc]").count();
        assert_snapshot!(
            "highlight_omnidoc_diff",
            format!(
                "Original omnidoc attrs: {}\nRestored omnidoc attrs: {}\nDiff: {}",
                original_omnidoc,
                restored_omnidoc,
                restored_omnidoc as i32 - original_omnidoc as i32
            )
        );
    }
}

#[test]
fn roundtrip_section() {
    let temp = setup_test_project(&[ModuleConfig::new("section")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{lib.rs,section.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"");

    result.snapshot_source_files("section");
    result.snapshot_docs_files("section");

    assert_snapshot!(
        "section_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (major restoration issues)"
        }
    );
}

#[test]
fn roundtrip_ui() {
    let temp = setup_test_project(&[ModuleConfig::new("ui")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{lib.rs,ui.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"{lib.md,ui.md,ui/{draw.md,draw_detail.md,draw_list.md,draw_list_with_command.md,get_tree_prefix.md}}");

    result.snapshot_source_files("ui");
    result.snapshot_docs_files("ui");

    assert_snapshot!(
        "ui_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (major restoration issues)"
        }
    );
}

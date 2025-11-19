#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_app_state() {
    let temp = setup_test_project(&[ModuleConfig::new("app_state")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{app_state,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{app_state,app_state/{AppState,AppState/{build_tree,cancel_move,command_buffer,cumulative_offset,current_node_index,current_view,editor_state,enter_detail_view,exit_detail_view,file_mode,file_offsets,files,find_next_node,find_prev_node,generate_edit_plan,get_current_section,get_current_section_index,get_indent,get_max_line_width,load_docs,mark_moved,message,move_section_down,move_section_in,move_section_out,move_section_to_bottom,move_section_to_top,move_section_up,move_state,moving_section_index,navigate_to_first,navigate_to_first_at_level,navigate_to_first_child,navigate_to_last,navigate_to_last_at_level,navigate_to_next_descendant,navigate_to_next_sibling,navigate_to_parent,navigate_to_prev_sibling,new,rebuild_file_offsets,rebuild_tree,rewrite_file_sections,save_current,save_section_reorder,sections,start_move,tree_nodes,wrap_width},FileMode,FileMode/{Multi,Single},MoveState,MoveState/{Moved,None,Selected},View,View/{Command,Detail,List}},lib}.md");

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
    assert_snapshot!(result.get_docs_files_brace(), @"{highlight,highlight/{SYNTAX_SET,THEME_SET,highlight_line_with_extension,highlight_source_lines},lib}.md");

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

    assert_snapshot!(result.get_source_files_brace(), @"{lib,ui}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{lib,ui,ui/{draw,draw_detail,draw_list,draw_list_with_command,get_tree_prefix}}.md");

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

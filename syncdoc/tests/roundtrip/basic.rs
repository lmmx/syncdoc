#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_config() {
    let temp = setup_test_project(&[ModuleConfig::new("config")]);
    let result = run_roundtrip(temp.path());

    // Snapshot file lists
    assert_snapshot!(result.get_source_files_brace(), @"{config,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{config.md,config/{Config.md,Config/{file_extensions.md,load.md,wrap_width.md}},lib.md}");

    // Snapshot file contents in subdirectory
    result.snapshot_source_files("config");
    result.snapshot_docs_files("config");

    // Document round-trip status
    assert_snapshot!(
        "config_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_edit_plan() {
    let temp = setup_test_project(&[ModuleConfig::new("edit_plan")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{edit_plan,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{edit_plan.md,edit_plan/{Edit.md,Edit/{column_end.md,column_start.md,file_name.md,item_name.md,line_end.md,line_start.md,section_content.md},EditPlan.md,EditPlan/{apply.md,edits.md}},lib.md}");

    result.snapshot_source_files("edit_plan");
    result.snapshot_docs_files("edit_plan");

    assert_snapshot!(
        "edit_plan_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_input() {
    let temp = setup_test_project(&[ModuleConfig::new("input")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{input,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{input.md,input/{build_hierarchy.md,extract_sections.md,find_documents.md,find_in_directory.md},lib.md}");

    result.snapshot_source_files("input");
    result.snapshot_docs_files("input");

    assert_snapshot!(
        "input_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_parent() {
    let temp = setup_test_project(&[ModuleConfig::new("formats")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats/{,Format/{,file_extension,format_section_display,language,section_query,title_query}},lib}.md");

    result.snapshot_source_files("formats_parent");
    result.snapshot_docs_files("formats_parent");

    assert_snapshot!(
        "formats_parent_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_markdown() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["markdown"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,formats/markdown.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats.md,formats/{Format.md,Format/{file_extension.md,format_section_display.md,language.md,section_query.md,title_query.md},markdown.md,markdown/{MarkdownFormat.md,MarkdownFormat/Format/{file_extension.md,format_section_display.md,language.md,section_query.md,title_query.md}}},lib.md,lib/formats.md}");

    result.snapshot_source_files("formats_markdown");
    result.snapshot_docs_files("formats_markdown");

    assert_snapshot!(
        "formats_markdown_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_difftastic() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["difftastic"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,formats/difftastic.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats.md,formats/{Format.md,Format/{file_extension.md,format_section_display.md,language.md,section_query.md,title_query.md},difftastic.md,difftastic/{DifftChange.md,DifftChange/{content.md,end.md,highlight.md,start.md},DifftFile.md,DifftFile/{chunks.md,language.md,path.md,status.md},DifftLine.md,DifftLine/{lhs.md,rhs.md},DifftSide.md,DifftSide/{changes.md,line_number.md},DifftasticFormat.md,DifftasticFormat/{Format/{file_extension.md,format_section_display.md,language.md,section_query.md,title_query.md},determine_hunk_color_from_header.md},create_chunk_section.md,extract_chunk_text.md,extract_column_range.md,extract_difftastic_sections.md,format_change_content.md,format_hunk_header.md,parse_difftastic_json.md}},lib.md,lib/formats.md}");

    result.snapshot_source_files("formats_difftastic");
    result.snapshot_docs_files("formats_difftastic");

    assert_snapshot!(
        "formats_difftastic_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

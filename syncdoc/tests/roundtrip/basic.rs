#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_config() {
    let temp = setup_test_project(&[ModuleConfig::new("config")]);
    let result = run_roundtrip(temp.path());

    // Snapshot file lists
    assert_snapshot!(result.get_source_files_brace(), @"{config,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{config,config/Config{,/{file_extensions,load,wrap_width}},lib}.md");

    // Snapshot file contents in subdirectory
    result.snapshot_source_files("config");
    result.snapshot_docs_files("config");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

#[test]
fn roundtrip_edit_plan() {
    let temp = setup_test_project(&[ModuleConfig::new("edit_plan")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{edit_plan,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{edit_plan,edit_plan/Edit{,/{column_end,column_start,file_name,item_name,line_end,line_start,section_content},Plan,Plan/{apply,edits}},lib}.md");

    result.snapshot_source_files("edit_plan");
    result.snapshot_docs_files("edit_plan");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

#[test]
fn roundtrip_input() {
    let temp = setup_test_project(&[ModuleConfig::new("input")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{input,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{input,input/{build_hierarchy,extract_sections,find_documents,find_in_directory},lib}.md");

    result.snapshot_source_files("input");
    result.snapshot_docs_files("input");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

#[test]
fn roundtrip_formats_parent() {
    let temp = setup_test_project(&[ModuleConfig::new("formats")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats,formats/Format{,/{file_extension,format_section_display,language,section_query,title_query}},lib}.md");

    result.snapshot_source_files("formats_parent");
    result.snapshot_docs_files("formats_parent");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

#[test]
fn roundtrip_formats_markdown() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["markdown"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats,formats/markdown,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats,formats/{Format,Format/{file_extension,format_section_display,language,section_query,title_query},markdown,markdown/MarkdownFormat{,/Format/{file_extension,format_section_display,language,section_query,title_query}}},lib,lib/formats}.md");

    result.snapshot_source_files("formats_markdown");
    result.snapshot_docs_files("formats_markdown");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

#[test]
fn roundtrip_formats_difftastic() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["difftastic"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats,formats/difftastic,lib}.rs");
    assert_snapshot!(result.get_docs_files_brace(), @"{formats,formats/{Format,Format/{file_extension,format_section_display,language,section_query,title_query},difftastic,difftastic/{DifftChange,DifftChange/{content,end,highlight,start},DifftFile,DifftFile/{chunks,language,path,status},DifftLine,DifftLine/{l,r}hs,DifftSide,DifftSide/{changes,line_number},DifftasticFormat,DifftasticFormat/{Format/{file_extension,format_section_display,language,section_query,title_query},determine_hunk_color_from_header},create_chunk_section,extract_chunk_text,extract_column_range,extract_difftastic_sections,format_change_content,format_hunk_header,parse_difftastic_json}},lib,lib/formats}.md");

    result.snapshot_source_files("formats_difftastic");
    result.snapshot_docs_files("formats_difftastic");

    assert!(
        result.is_perfectly_restored(),
        "Round-trip failed: original != restored"
    );
}

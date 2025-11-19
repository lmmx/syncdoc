//! Syntax highlighting for displaying source code with visual distinction between language elements.
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: std::sync::LazyLock<SyntaxSet> =
    std::sync::LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: std::sync::LazyLock<ThemeSet> = std::sync::LazyLock::new(ThemeSet::load_defaults);

/// Highlight source lines with syntax coloring.
///
/// # Panics
///
/// Panics if syntax highlighting fails for any line in the input.
pub fn highlight_source_lines(
    lines: &[&str],
    start: usize,
    end: usize,
    target_line: usize,
) -> Vec<Line<'static>> {
    let theme = &THEME_SET.themes["base16-eighties.dark"];
    let syntax_ref = SYNTAX_SET
        .find_syntax_by_extension("rs")
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlight_lines = HighlightLines::new(syntax_ref, theme);
    let mut display_lines: Vec<Line<'static>> = Vec::new();

    for (i, line_text) in lines[start..end].iter().enumerate() {
        let line_num = start + i + 1;
        let marker = if line_num == target_line + 1 {
            "→"
        } else {
            " "
        };
        let line_num_text = format!("{marker} {line_num:4} | ");

        let highlighted = highlight_lines
            .highlight_line(line_text, &SYNTAX_SET)
            .unwrap();

        let mut spans = vec![Span::raw(line_num_text)];
        for (style, text) in highlighted {
            spans.push(Span::styled(
                text.to_string(),
                Style::default().fg(Color::Rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                )),
            ));
        }

        display_lines.push(Line::from(spans));
    }

    display_lines
}

/// Highlight a single line with syntax coloring for a given file extension.
///
/// Falls back to plain text rendering if syntax highlighting fails.
pub fn highlight_line_with_extension(line: &str, extension: &str) -> Line<'static> {
    let theme = &THEME_SET.themes["base16-eighties.dark"];
    let syntax_ref = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlight_lines = HighlightLines::new(syntax_ref, theme);

    // Try to highlight, fall back to plain text if it fails
    match highlight_lines.highlight_line(line, &SYNTAX_SET) {
        Ok(highlighted) => {
            let mut spans = Vec::new();
            for (style, text) in highlighted {
                spans.push(Span::styled(
                    text.to_string(),
                    Style::default().fg(Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    )),
                ));
            }
            Line::from(spans)
        }
        Err(_) => {
            // Fall back to plain text if highlighting fails
            Line::from(line.to_string())
        }
    }
}

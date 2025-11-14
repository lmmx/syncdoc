use regex::Regex;
use std::sync::LazyLock;

pub fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

pub static FN_RE: LazyLock<Regex> = LazyLock::new(|| re(r"\bfn\s+(\w+)\s*[<(]"));
pub static STRUCT_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?struct\s+(\w+)"));
pub static ENUM_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?enum\s+(\w+)"));
pub static CONST_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?const\s+(\w+)\s*:"));
pub static TYPE_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?type\s+(\w+)"));
pub static IMPL_RE: LazyLock<Regex> =
    LazyLock::new(|| re(r"impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)"));
pub static TRAIT_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?trait\s+(\w+)"));
pub static MOD_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:pub\s+)?mod\s+(\w+)\s*\{"));
pub static FIELD_RE: LazyLock<Regex> = LazyLock::new(|| re(r"^\s*(?:pub\s+)?(\w+)\s*:"));

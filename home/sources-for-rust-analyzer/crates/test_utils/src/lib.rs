//! Assorted testing utilities.
//!
//! Most notable things are:
//!
//! * Rich text comparison, which outputs a diff.
//! * Extracting markup (mainly, `<|>` markers) out of fixture strings.
//! * marks (see the eponymous module).

#[macro_use]
pub mod mark;
mod fixture;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use text_size::{TextRange, TextSize};

pub use difference::Changeset as __Changeset;
pub use rustc_hash::FxHashMap;

pub use crate::fixture::Fixture;

pub const CURSOR_MARKER: &str = "<|>";

/// Asserts that two strings are equal, otherwise displays a rich diff between them.
///
/// The diff shows changes from the "original" left string to the "actual" right string.
///
/// All arguments starting from and including the 3rd one are passed to
/// `eprintln!()` macro in case of text inequality.
#[macro_export]
macro_rules! assert_eq_text {
    ($left:expr, $right:expr) => {
        assert_eq_text!($left, $right,)
    };
    ($left:expr, $right:expr, $($tt:tt)*) => {{
        let left = $left;
        let right = $right;
        if left != right {
            if left.trim() == right.trim() {
                eprintln!("Left:\n{:?}\n\nRight:\n{:?}\n\nWhitespace difference\n", left, right);
            } else {
                let changeset = $crate::__Changeset::new(left, right, "\n");
                eprintln!("Left:\n{}\n\nRight:\n{}\n\nDiff:\n{}\n", left, right, changeset);
            }
            eprintln!($($tt)*);
            panic!("text differs");
        }
    }};
}

/// Infallible version of `try_extract_offset()`.
pub fn extract_offset(text: &str) -> (TextSize, String) {
    match try_extract_offset(text) {
        None => panic!("text should contain cursor marker"),
        Some(result) => result,
    }
}

/// Returns the offset of the first occurence of `<|>` marker and the copy of `text`
/// without the marker.
fn try_extract_offset(text: &str) -> Option<(TextSize, String)> {
    let cursor_pos = text.find(CURSOR_MARKER)?;
    let mut new_text = String::with_capacity(text.len() - CURSOR_MARKER.len());
    new_text.push_str(&text[..cursor_pos]);
    new_text.push_str(&text[cursor_pos + CURSOR_MARKER.len()..]);
    let cursor_pos = TextSize::from(cursor_pos as u32);
    Some((cursor_pos, new_text))
}

/// Infallible version of `try_extract_range()`.
pub fn extract_range(text: &str) -> (TextRange, String) {
    match try_extract_range(text) {
        None => panic!("text should contain cursor marker"),
        Some(result) => result,
    }
}

/// Returns `TextRange` between the first two markers `<|>...<|>` and the copy
/// of `text` without both of these markers.
fn try_extract_range(text: &str) -> Option<(TextRange, String)> {
    let (start, text) = try_extract_offset(text)?;
    let (end, text) = try_extract_offset(&text)?;
    Some((TextRange::new(start, end), text))
}

#[derive(Clone, Copy)]
pub enum RangeOrOffset {
    Range(TextRange),
    Offset(TextSize),
}

impl From<RangeOrOffset> for TextRange {
    fn from(selection: RangeOrOffset) -> Self {
        match selection {
            RangeOrOffset::Range(it) => it,
            RangeOrOffset::Offset(it) => TextRange::empty(it),
        }
    }
}

/// Extracts `TextRange` or `TextSize` depending on the amount of `<|>` markers
/// found in `text`.
///
/// # Panics
/// Panics if no `<|>` marker is present in the `text`.
pub fn extract_range_or_offset(text: &str) -> (RangeOrOffset, String) {
    if let Some((range, text)) = try_extract_range(text) {
        return (RangeOrOffset::Range(range), text);
    }
    let (offset, text) = extract_offset(text);
    (RangeOrOffset::Offset(offset), text)
}

/// Extracts ranges, marked with `<tag> </tag>` pairs from the `text`
pub fn extract_ranges(mut text: &str, tag: &str) -> (Vec<TextRange>, String) {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let mut ranges = Vec::new();
    let mut res = String::new();
    let mut stack = Vec::new();
    loop {
        match text.find('<') {
            None => {
                res.push_str(text);
                break;
            }
            Some(i) => {
                res.push_str(&text[..i]);
                text = &text[i..];
                if text.starts_with(&open) {
                    text = &text[open.len()..];
                    let from = TextSize::of(&res);
                    stack.push(from);
                } else if text.starts_with(&close) {
                    text = &text[close.len()..];
                    let from = stack.pop().unwrap_or_else(|| panic!("unmatched </{}>", tag));
                    let to = TextSize::of(&res);
                    ranges.push(TextRange::new(from, to));
                }
            }
        }
    }
    assert!(stack.is_empty(), "unmatched <{}>", tag);
    ranges.sort_by_key(|r| (r.start(), r.end()));
    (ranges, res)
}

/// Inserts `<|>` marker into the `text` at `offset`.
pub fn add_cursor(text: &str, offset: TextSize) -> String {
    let offset: usize = offset.into();
    let mut res = String::new();
    res.push_str(&text[..offset]);
    res.push_str("<|>");
    res.push_str(&text[offset..]);
    res
}

// Comparison functionality borrowed from cargo:

/// Compare a line with an expected pattern.
/// - Use `[..]` as a wildcard to match 0 or more characters on the same line
///   (similar to `.*` in a regex).
pub fn lines_match(expected: &str, actual: &str) -> bool {
    // Let's not deal with / vs \ (windows...)
    // First replace backslash-escaped backslashes with forward slashes
    // which can occur in, for example, JSON output
    let expected = expected.replace(r"\\", "/").replace(r"\", "/");
    let mut actual: &str = &actual.replace(r"\\", "/").replace(r"\", "/");
    for (i, part) in expected.split("[..]").enumerate() {
        match actual.find(part) {
            Some(j) => {
                if i == 0 && j != 0 {
                    return false;
                }
                actual = &actual[j + part.len()..];
            }
            None => return false,
        }
    }
    actual.is_empty() || expected.ends_with("[..]")
}

#[test]
fn lines_match_works() {
    assert!(lines_match("a b", "a b"));
    assert!(lines_match("a[..]b", "a b"));
    assert!(lines_match("a[..]", "a b"));
    assert!(lines_match("[..]", "a b"));
    assert!(lines_match("[..]b", "a b"));

    assert!(!lines_match("[..]b", "c"));
    assert!(!lines_match("b", "c"));
    assert!(!lines_match("b", "cb"));
}

/// Compares JSON object for approximate equality.
/// You can use `[..]` wildcard in strings (useful for OS dependent things such
/// as paths). You can use a `"{...}"` string literal as a wildcard for
/// arbitrary nested JSON. Arrays are sorted before comparison.
pub fn find_mismatch<'a>(expected: &'a Value, actual: &'a Value) -> Option<(&'a Value, &'a Value)> {
    use serde_json::Value::*;
    match (expected, actual) {
        (&Number(ref l), &Number(ref r)) if l == r => None,
        (&Bool(l), &Bool(r)) if l == r => None,
        (&String(ref l), &String(ref r)) if lines_match(l, r) => None,
        (&Array(ref l), &Array(ref r)) => {
            if l.len() != r.len() {
                return Some((expected, actual));
            }

            let mut l = l.iter().collect::<Vec<_>>();
            let mut r = r.iter().collect::<Vec<_>>();

            l.retain(|l| match r.iter().position(|r| find_mismatch(l, r).is_none()) {
                Some(i) => {
                    r.remove(i);
                    false
                }
                None => true,
            });

            if !l.is_empty() {
                assert!(!r.is_empty());
                Some((&l[0], &r[0]))
            } else {
                assert_eq!(r.len(), 0);
                None
            }
        }
        (&Object(ref l), &Object(ref r)) => {
            let same_keys = l.len() == r.len() && l.keys().all(|k| r.contains_key(k));
            if !same_keys {
                return Some((expected, actual));
            }

            l.values().zip(r.values()).filter_map(|(l, r)| find_mismatch(l, r)).next()
        }
        (&Null, &Null) => None,
        // magic string literal "{...}" acts as wildcard for any sub-JSON
        (&String(ref l), _) if l == "{...}" => None,
        _ => Some((expected, actual)),
    }
}

/// Calls callback `f` with input code and file paths for each `.rs` file in `test_data_dir`
/// subdirectories defined by `paths`.
///
/// If the content of the matching output file differs from the output of `f()`
/// the test will fail.
///
/// If there is no matching output file it will be created and filled with the
/// output of `f()`, but the test will fail.
pub fn dir_tests<F>(test_data_dir: &Path, paths: &[&str], outfile_extension: &str, f: F)
where
    F: Fn(&str, &Path) -> String,
{
    for (path, input_code) in collect_rust_files(test_data_dir, paths) {
        let actual = f(&input_code, &path);
        let path = path.with_extension(outfile_extension);
        if !path.exists() {
            println!("\nfile: {}", path.display());
            println!("No .txt file with expected result, creating...\n");
            println!("{}\n{}", input_code, actual);
            fs::write(&path, &actual).unwrap();
            panic!("No expected result");
        }
        let expected = read_text(&path);
        assert_equal_text(&expected, &actual, &path);
    }
}

/// Collects all `.rs` files from `dir` subdirectories defined by `paths`.
pub fn collect_rust_files(root_dir: &Path, paths: &[&str]) -> Vec<(PathBuf, String)> {
    paths
        .iter()
        .flat_map(|path| {
            let path = root_dir.to_owned().join(path);
            rust_files_in_dir(&path).into_iter()
        })
        .map(|path| {
            let text = read_text(&path);
            (path, text)
        })
        .collect()
}

/// Collects paths to all `.rs` files from `dir` in a sorted `Vec<PathBuf>`.
fn rust_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut acc = Vec::new();
    for file in fs::read_dir(&dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        if path.extension().unwrap_or_default() == "rs" {
            acc.push(path);
        }
    }
    acc.sort();
    acc
}

/// Returns the path to the root directory of `rust-analyzer` project.
pub fn project_dir() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(dir).parent().unwrap().parent().unwrap().to_owned()
}

/// Read file and normalize newlines.
///
/// `rustc` seems to always normalize `\r\n` newlines to `\n`:
///
/// ```
/// let s = "
/// ";
/// assert_eq!(s.as_bytes(), &[10]);
/// ```
///
/// so this should always be correct.
pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("File at {:?} should be valid", path))
        .replace("\r\n", "\n")
}

/// Returns `false` if slow tests should not run, otherwise returns `true` and
/// also creates a file at `./target/.slow_tests_cookie` which serves as a flag
/// that slow tests did run.
pub fn skip_slow_tests() -> bool {
    let should_skip = std::env::var("CI").is_err() && std::env::var("RUN_SLOW_TESTS").is_err();
    if should_skip {
        eprintln!("ignoring slow test")
    } else {
        let path = project_dir().join("./target/.slow_tests_cookie");
        fs::write(&path, ".").unwrap();
    }
    should_skip
}

/// Asserts that `expected` and `actual` strings are equal. If they differ only
/// in trailing or leading whitespace the test won't fail and
/// the contents of `actual` will be written to the file located at `path`.
fn assert_equal_text(expected: &str, actual: &str, path: &Path) {
    if expected == actual {
        return;
    }
    let dir = project_dir();
    let pretty_path = path.strip_prefix(&dir).unwrap_or_else(|_| path);
    if expected.trim() == actual.trim() {
        println!("whitespace difference, rewriting");
        println!("file: {}\n", pretty_path.display());
        fs::write(path, actual).unwrap();
        return;
    }
    if env::var("UPDATE_EXPECTATIONS").is_ok() {
        println!("rewriting {}", pretty_path.display());
        fs::write(path, actual).unwrap();
        return;
    }
    assert_eq_text!(expected, actual, "file: {}", pretty_path.display());
}

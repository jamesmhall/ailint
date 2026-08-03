use ailint_extractor::{extract, Comment, CommentKind, Language};

fn kinds(comments: &[Comment]) -> Vec<CommentKind> {
    comments.iter().map(|c| c.kind).collect()
}

fn bodies(comments: &[Comment]) -> Vec<String> {
    comments
        .iter()
        .map(|c| c.body().trim().to_string())
        .collect()
}

fn lines(comments: &[Comment]) -> Vec<usize> {
    comments.iter().map(|c| c.line).collect()
}

// ---------- Rust ----------

#[test]
fn rust_line_and_block_comments_extracted() {
    let src = "\
fn main() {
    // line one
    /* block */
    let x = 1;
}
";
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 2);
    assert_eq!(kinds(&cs), vec![CommentKind::Line, CommentKind::Block]);
    assert_eq!(bodies(&cs), vec!["line one", "block"]);
    assert_eq!(lines(&cs), vec![2, 3]);
}

#[test]
fn rust_doc_comments_classified() {
    let src = "\
/// outer doc
//! inner doc
/** block outer */
/*! block inner */
//// not a doc (four slashes)
";
    let cs = extract(src, Language::Rust);
    assert_eq!(
        kinds(&cs),
        vec![
            CommentKind::Doc,
            CommentKind::Doc,
            CommentKind::Doc,
            CommentKind::Doc,
            CommentKind::Line,
        ]
    );
}

#[test]
fn rust_ignores_double_slash_inside_string() {
    let src = r#"let s = "not // a comment"; // real"#;
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn rust_ignores_star_slash_inside_string() {
    let src = r#"let s = "/* not a block */"; /* real */"#;
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].kind, CommentKind::Block);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn rust_ignores_comment_markers_in_raw_strings() {
    let src = r####"
let a = r"//not a comment";
let b = r#"also // not one"#;
let c = r##"still /* not */ one"##;
// real
"####;
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn rust_char_literals_do_not_open_strings() {
    // `'"'` is a char literal; the double-quote inside must not open a string.
    let src = "let q = '\"'; // trailing";
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "trailing");
}

#[test]
fn rust_byte_and_c_strings_are_consumed() {
    let src = r#"
let b = b"// not a comment";
// real
"#;
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn rust_byte_ranges_slice_back_to_source() {
    let src = "// hello\nfn f() {}\n";
    let cs = extract(src, Language::Rust);
    assert_eq!(cs.len(), 1);
    assert_eq!(&src[cs[0].byte_range.clone()], "// hello");
}

// ---------- TypeScript / JavaScript ----------

#[test]
fn ts_line_block_and_jsdoc() {
    let src = "\
// hi
/* block */
/** jsdoc */
const x = 1;
";
    let cs = extract(src, Language::TypeScript);
    assert_eq!(
        kinds(&cs),
        vec![CommentKind::Line, CommentKind::Block, CommentKind::Doc,]
    );
    assert_eq!(bodies(&cs), vec!["hi", "block", "jsdoc"]);
}

#[test]
fn ts_ignores_slashes_in_strings_and_template_literals() {
    let src = r#"
const s = "// not";
const t = '/* not */';
const tpl = `also // not /* not */ real`;
// real
"#;
    let cs = extract(src, Language::JavaScript);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn ts_double_star_block_is_not_docstring_when_empty_pair() {
    // `/**/` is empty block, not a doc comment. `/***/` starts with `/**`
    // but has three stars — Rust convention (and Rustdoc) says not a doc,
    // and we mirror that in JS/TS to avoid a lonely `/***/ ` false positive.
    let src = "/**/\n/***/\n/** real */\n";
    let cs = extract(src, Language::JavaScript);
    assert_eq!(
        kinds(&cs),
        vec![CommentKind::Block, CommentKind::Block, CommentKind::Doc,]
    );
}

#[test]
fn ts_multiline_block_line_number_is_start() {
    let src = "\
const x = 1;
/*
 * multi
 */
const y = 2;
";
    let cs = extract(src, Language::TypeScript);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].line, 2);
}

// ---------- Python ----------

#[test]
fn python_hash_and_docstrings_extracted() {
    let src = "\
def f():
    \"\"\"docstring\"\"\"
    # comment
    return 1
";
    let cs = extract(src, Language::Python);
    assert_eq!(cs.len(), 2);
    assert_eq!(kinds(&cs), vec![CommentKind::Docstring, CommentKind::Line]);
    assert_eq!(bodies(&cs), vec!["docstring", "comment"]);
}

#[test]
fn python_ignores_hash_inside_string() {
    let src = "s = \"# not\"\n# real\n";
    let cs = extract(src, Language::Python);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].body().trim(), "real");
}

#[test]
fn python_triple_single_quoted_docstring() {
    let src = "'''single triple'''\n";
    let cs = extract(src, Language::Python);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].kind, CommentKind::Docstring);
    assert_eq!(cs[0].body().trim(), "single triple");
}

#[test]
fn python_multiline_docstring_span_and_line() {
    let src = "\
def f():
    \"\"\"
    multi
    line
    \"\"\"
    pass
";
    let cs = extract(src, Language::Python);
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].kind, CommentKind::Docstring);
    assert_eq!(cs[0].line, 2);
    assert!(cs[0].body().contains("multi"));
    assert!(cs[0].body().contains("line"));
}

#[test]
fn python_unterminated_triple_string_is_skipped_gracefully() {
    let src = "\"\"\"unterminated\n# would-be comment\n";
    let cs = extract(src, Language::Python);
    // Logos returns an Err for the unclosed triple string; we ignore errors
    // and keep scanning. The important guarantee is no panic and that every
    // returned comment slice still points inside the source.
    for c in &cs {
        assert!(c.byte_range.end <= src.len());
        assert!(!c.body().is_empty() || c.raw.is_empty());
    }
}

// ---------- Cross-cutting ----------

#[test]
fn multiple_comments_preserve_source_order() {
    let src = "// a\n// b\n// c\n";
    let cs = extract(src, Language::Rust);
    assert_eq!(bodies(&cs), vec!["a", "b", "c"]);
}

// ---------- Go ----------

#[test]
fn go_line_and_block_comments_extracted() {
    let src = "\
package main

// line one
/* block */
func main() {}
";
    let cs = extract(src, Language::Go);
    assert_eq!(kinds(&cs), vec![CommentKind::Line, CommentKind::Block]);
    assert_eq!(bodies(&cs), vec!["line one", "block"]);
    assert_eq!(lines(&cs), vec![3, 4]);
}

#[test]
fn go_ignores_slashes_in_strings_and_raw_strings() {
    let src = r#"
package main

const a = "not // a comment"
const b = `raw // not a comment`
// real
"#;
    let cs = extract(src, Language::Go);
    assert_eq!(bodies(&cs), vec!["real"]);
}

#[test]
fn go_rune_literal_does_not_open_string() {
    let src = "\
package main
const q = '/'
// real
";
    let cs = extract(src, Language::Go);
    assert_eq!(bodies(&cs), vec!["real"]);
}

// ---------- Java ----------

#[test]
fn java_line_block_and_javadoc() {
    let src = "\
// line one
/* block */
/** javadoc */
class C {}
";
    let cs = extract(src, Language::Java);
    assert_eq!(
        kinds(&cs),
        vec![CommentKind::Line, CommentKind::Block, CommentKind::Doc]
    );
    assert_eq!(bodies(&cs), vec!["line one", "block", "javadoc"]);
}

#[test]
fn java_ignores_slashes_in_strings_and_text_blocks() {
    let src = "\
class C {
    String a = \"not // a comment\";
    String b = \"\"\"
        text block // not a comment
        \"\"\";
    // real
}
";
    let cs = extract(src, Language::Java);
    assert_eq!(bodies(&cs), vec!["real"]);
}

// ---------- C# ----------

#[test]
fn cs_line_block_and_xmldoc() {
    let src = "\
// line one
/// xml doc
/* block */
//// four slashes is not a doc
class C {}
";
    let cs = extract(src, Language::CSharp);
    assert_eq!(
        kinds(&cs),
        vec![
            CommentKind::Line,
            CommentKind::Doc,
            CommentKind::Block,
            CommentKind::Line,
        ]
    );
}

#[test]
fn cs_ignores_slashes_in_verbatim_and_raw_strings() {
    let src = "\
class C {
    string a = @\"not // a comment with \"\"escapes\"\"\";
    string b = \"\"\"raw // not a comment\"\"\";
    string c = $\"not // a comment {x}\";
    // real
}
";
    let cs = extract(src, Language::CSharp);
    assert_eq!(bodies(&cs), vec!["real"]);
}

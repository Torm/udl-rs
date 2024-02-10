use std::ops::Deref;
use khi::parse::{parse_dictionary_str, parse_expression_str, parse_table_str, ParsedValue};
use khi::{Composition, Dictionary, Tag, Value, Table, Element};

#[test]
fn test_lexer() { // TODO

}

#[test]
fn test_terms() {
    let source = "A text term";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_text());
    let source = "{k1: v1; k2: v2}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_dictionary());
    let source = "[e1; e2; e3]";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_table());
    let source = "<Pattern>:arg:arg";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_tag());
}

#[test]
fn test_expressions() {

}

#[test]
fn test_grouping() {
    let source = "{A text term}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_text());
}

#[test]
fn test_text_terms() {
    assert_text("Hello world!", "Hello world!");
    assert_text(" Hello world! ", "Hello world!");
    assert_text("Hello\tworld!", "Hello world!");
    assert_text("Hello\nworld!", "Hello world!");
    assert_text("R e d", "R e d");
    assert_text("R ~ e ~ d", "Red");
    assert_text("A<#>A  A<#>. A\\A  A\\.", "AA  A. AA  A.");
}

#[test]
fn test_pattern_composition() {
    let source = "<p1>:arg1:arg2: <p3>:arg4:arg5: <p6>: <p7>:arg8";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_tag());
    let p1 = document.as_tag().unwrap();
    assert_eq!(p1.get().unfold().len(), 3);
    let p1arg = p1.get().unfold();
    let p1arg = p1arg.get(2).unwrap();
    assert!(p1arg.is_tag());
    let p2 = p1arg.as_tag().unwrap();
    assert_eq!(p2.get().unfold().len(), 3);
    let p2arg = p2.get().unfold();
    let p2arg = p2arg.get(2).unwrap();
    assert!(p2arg.is_tag());
    let p3 = p2arg.as_tag().unwrap().get();
    assert!(p3.is_tag());
}

#[test]
fn test_expression() {
    assert_terms("", "");
    assert_terms("Text", "Tx");
    assert_terms("{k: v}", "Dc");
    assert_terms("[1|0;0|1]", "Tb");
    assert_terms("<P>", "Pt");
    assert_terms("{~} {Text [Table]}", "Nl Cm");
    assert_terms("{Text [Table]}", "Tx Tb");
    assert_terms("{~}", "");
    assert_terms("Text {Text} [Table] {k: v} <Dir>", "Tx Tx Tb Dc Pt");
    assert_terms("Text \"Text\" [Table] {k: v} <Dir>", "Tx Tb Dc Pt");
}

#[test]
fn test_tuple() {
    assert_tuple("a a :: b b", 2);
    assert_tuple("a :: [b] :: {c}", 3);
    assert_tuple("<a>:b:c :: d d :: <e>:f", 3);
    assert_pattern("<a> :: b", 1);
    assert_pattern("<a> :: b b :: c", 2);
}

fn assert_tuple(source: &str, len: usize) {
    let expression = parse_expression_str(source).unwrap();
    let tuple = expression.unfold();
    assert_eq!(tuple.len(), len);
}

fn assert_pattern(source: &str, len: usize) {
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_tag());
    let pattern = document.as_tag().unwrap();
    assert_eq!(pattern.get().unfold().len(), len);
}

fn assert_terms(source: &str, expect: &str) {
    let document = parse_expression_str(source).unwrap();
    let summary = summarize_terms(&document);
    assert!(summary.eq(expect));
}

fn summarize_terms(value: &ParsedValue) -> String {
    let mut summary = String::new();
    match value {
        ParsedValue::Nil(..) => summary = format!("{}", summary),
        ParsedValue::Text(..) => summary = format!("{}Tx", summary),
        ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
        ParsedValue::Tuple(..) => summary = format!("{}Tp", summary),
        ParsedValue::Table(..) => summary = format!("{}Tb", summary),
        ParsedValue::Composition(composition, ..) => {
            for element in composition.iter() {
                match element {
                    Element::Solid(c) => {
                        match c {
                            ParsedValue::Nil(..) => summary = format!("{}Nl", summary),
                            ParsedValue::Text(..) => summary = format!("{}Tx", summary),
                            ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
                            ParsedValue::Tuple(..) => summary = format!("{}Tp", summary),
                            ParsedValue::Table(..) => summary = format!("{}Tb", summary),
                            ParsedValue::Composition(..) => summary = format!("{}Cm", summary),
                            ParsedValue::Tag(..) => summary = format!("{}Pt", summary),
                        }
                    }
                    Element::Space => summary = format!("{} ", summary),
                }
            }
        }
        ParsedValue::Tag(..) => summary = format!("{}Pt", summary),
    }
    summary
}

#[test]
fn test_escape_sequences() {
    assert_text(":`", ":");
    assert_text(";`", ";");
    assert_text("|`", "|");
    assert_text("~`", "~");
    assert_text("``", "`");
    assert_text("\\`", "\\");
    assert_text("{`", "{");
    assert_text("}`", "}");
    assert_text("[`", "[");
    assert_text("]`", "]");
    assert_text("<`", "<");
    assert_text(">`", ">");
    assert_text("#`", "#");
    assert_text("n`", "\n");
    // Invalid escapes
    assert_invalid_expression("a`");
    assert_invalid_expression("1`");
    assert_invalid_expression("`[");
    assert_invalid_expression("`n");
}

#[test]
fn test_repeated_escape_sequences() {
    assert_terms("||", "Tx");
    assert_terms("<<", "Tx");
    assert_terms("<<<", "Tx");
    assert_terms(">>", "Tx");
    assert_terms(">>>", "Tx");
}

#[test]
fn test_hash() {
    // Text
    assert_terms("#a", "Tx");
    assert_terms("#1", "Tx");
    assert_terms("#?", "Tx");
    assert_terms("#:`", "Tx");
    assert_terms("#>>", "Tx");
    assert_terms("A#B", "Tx");
    // Comments
    assert_terms("##", "");
    assert_terms("# ", "");
    assert_terms("#", "");
    // Invalid sequence
    assert_invalid_expression("#{");
    assert_invalid_expression("#}");
    assert_invalid_expression("#[");
    assert_invalid_expression("#]");
    assert_invalid_expression("#<");
    assert_invalid_expression("#>");
    assert_invalid_expression("#\\");
    assert_invalid_expression("#:");
    assert_invalid_expression("#;");
    assert_invalid_expression("#|");
    assert_invalid_expression("#~");
}

fn assert_invalid_expression(source: &str) {
    assert!(parse_expression_str(source).is_err());
}

#[test]
fn test_table() {
    // Test empty table
    let document = parse_expression_str("[]").unwrap();
    assert!(document.is_table());
    let table = document.as_table().unwrap();
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
    // Test valid sequential notation.
    assert_table("", 0, 0);
    assert_table("1", 1, 1);
    assert_table("~", 1, 1);
    assert_table("1;", 1, 1);
    assert_table("~;", 1, 1);
    assert_table("1|0", 1, 2);
    assert_table("~|~", 1, 2);
    assert_table("1|0;", 1, 2);
    assert_table("1;0", 2, 1);
    assert_table("~;~", 2, 1);
    assert_table("1;0;", 2, 1);
    assert_table("1|0;0|1", 2, 2);
    assert_table("1|0|0;0|1|0;0|0|1", 3, 3);
    assert_table("1|~|~;~|1|~;~|~|1", 3, 3);
    assert_table("~|~|~;~|~|~;~|~|~", 3, 3);
    // Test invalid sequential notation.
    assert_invalid_table("1|0; ;");
    assert_invalid_table(";1|0;");
    assert_invalid_table("1|0|");
    assert_invalid_table("1|~|");
    assert_invalid_table("1| |0");
    // Test valid tabular notation.
    assert_table("|1|", 1, 1);
    assert_table("|~|", 1, 1);
    assert_table("|1|1|", 1, 2);
    assert_table("|~|~|", 1, 2);
    assert_table("|1|0| |0|1|", 2, 2);
    assert_table("|~|~| |~|~|", 2, 2);
    assert_table("|1|~| |~|1|", 2, 2);
    assert_table("|1| |1|", 2, 1);
    assert_table("|1|0|0| |0|1|0| |0|0|1|", 3, 3);
    // Test invalid tabular notation.
    assert_invalid_table("|");
    assert_invalid_table("| |");
    assert_invalid_table("|a");
    assert_invalid_table("|a|b|c| |d|e|f");
    assert_invalid_table("|a|b|c| | |d|e|f");
    assert_invalid_table("|a|b|c| |d|e|");
    assert_invalid_table("|a|b| |d|e|f|");
    assert_invalid_table("|a|b| |~|e|f|");
    // Test valid bullet notation.
    assert_table("> A > B > C", 3, 1);
    assert_table("> A | B | C", 1, 3);
    assert_table("> A | B > C | D", 2, 2);
    // Test invalid bullet notation.
    assert_invalid_table("> > A > B > C");
    assert_invalid_table("> > A > B > > C");
    assert_invalid_table("> | A > B > C");
    assert_invalid_table("> A > B > | C");
}

fn assert_table(source: &str, rows: usize, columns: usize) {
    let document = parse_table_str(source).unwrap();
    assert_eq!(document.rows(), rows);
    assert_eq!(document.columns(), columns);
    assert_eq!(document.len(), rows * columns);
}

fn assert_invalid_table(source: &str) {
    let document = parse_table_str(source);
    assert!(document.is_err());
}

#[test]
fn test_transcription() {
    assert_text("\\a b c\\", "a b c");
    assert_text("\\ a b  c d  e f  g h \\", " a b  c d  e f  g h ");
    assert!(parse_expression_str("\\a b\nc d\\").is_err());
}

#[test]
fn test_default_text_block() {
    let expect = "def main():\n  print(\"Hello world\")\nmain()\n";
    // Test indentation.
    let source = "<#>\n  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
    // Equal increase in indentation.
    let source = "<#>\n    def main():\n      print(\"Hello world\")\n    main()\n<#>";
    assert_text(source, expect);
    // Start immediately after <#>.
    let source = "<#>  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
}

#[test]
fn test_text_block_configuration() {
    let source = "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ";
    assert_text(
        &format!("{}{}{}", "<#>", source, "<#>"),
        "level 1  \nlevel 1\n  level 2  \n  level 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# r>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rf>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rh>", source, "<#>"),
        "  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rx>", source, "<#>"),
        "\nlevel 1  \nlevel 1\n  level 2  \n  level 2\nlevel 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rt>", source, "<#>"),
        "\n  level 1\n  level 1\n    level 2\n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rl>", source, "<#>"),
        "\nlevel 1  \nlevel 1\nlevel 2  \nlevel 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rn>", source, "<#>"),
        "    level 1    level 1    level 2      level 2  level 1  ",
    );
}

fn assert_text(source: &str, expect: &str) {
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_text());
    let string = document.as_text().unwrap().str.deref();
    assert_eq!(string, expect);
}

#[test]
fn test_join_operator() {
    assert_terms("~", "");
    assert_terms("~ ~", "");
    assert_terms("A ~", "Tx");
    assert_terms("A ~ B", "Tx");
    assert_terms("A ~ B ~ C", "Tx");
    assert_terms("{A} ~ {B}", "TxTx");
    assert_terms("{A} ~ {B} ~ {C}", "TxTxTx");
    assert_terms("{A}~{B}~{C}", "TxTxTx");
    assert_terms("~{A} {B}~ ~{C}~", "Tx TxTx");
}

#[test]
fn test_dictionary() {
    // Test empty
    let source = "{}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 0);
    assert!(dictionary.is_empty());
    // Test regular
    let source = "{k1: v1; k2: v2; k3: v3}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 3);
    // Test trailing
    let source = "{k1: v1; k2: v2; k3: v3;}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 3);
    assert_dictionary("", 0);
    assert_dictionary("k:v", 1);
    assert_dictionary("k:v;", 1);
    assert_dictionary("k:~", 1);
    assert_dictionary("k1:v1;k2:v2", 2);
    assert_dictionary("k1:v1;k2:v2;", 2);
    assert_dictionary("k1:~;k2:v2", 2);
    assert_dictionary("k1:~;k2:~", 2);
    assert_dictionary("k1:~;k2:~;", 2);
}

fn assert_dictionary(source: &str, size: usize) {
    let document = parse_dictionary_str(source).unwrap();
    assert_eq!(document.len(), size);
}

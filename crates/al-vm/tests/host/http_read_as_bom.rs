//! `HttpContent.ReadAs(Text)` of a body that starts with a UTF-8 byte order mark (EF BB BF), as Azure Blob Storage's
//! List Blobs answer does. Not pinned by the corpus yet: every expected value is what BC 29.0.54011.55616 answered to
//! the probe of assumption `http-readas-text-strips-byte-order-mark` (each case names its cell).

use al_builtins::native::Signal;
use al_builtins::value::Value;

const TABLE: &str = r#"
table 60009 "ALT Bom Blob"
{
    fields
    {
        field(1; "Code"; Code[20]) { }
        field(3; Data; Blob) { }
    }
    keys { key(PK; "Code") { } }
}
"#;

/// Writes `bytes` to a content through a stream, sets `content_type` (none when empty), reads it with `ReadAs(Text)` and
/// gives the text's length and character codes.
fn read(bytes: &[u8], content_type: &str) -> Result<Value, Signal> {
    let writes: String = bytes.iter().map(|b| format!("V := {b}; O.Write(V); ")).collect();
    let set_type = if content_type.is_empty() {
        String::new()
    } else {
        format!("C.GetHeaders(H); H.Remove('Content-Type'); H.Add('Content-Type', '{content_type}');")
    };
    let src = format!(
        "{TABLE}\ncodeunit 50100 C {{
         procedure F(): Text var B: Record \"ALT Bom Blob\"; O: OutStream; I: InStream; C: HttpContent; H: HttpHeaders; V: Byte;
         T: Text; Ch: Char; N: Integer; K: Integer; R: Text;
         begin B.Data.CreateOutStream(O); {writes} B.Data.CreateInStream(I); C.WriteFrom(I); {set_type}
         C.ReadAs(T); R := Format(StrLen(T)) + ':';
         for K := 1 to StrLen(T) do begin Ch := T[K]; N := Ch; R += Format(N) + ','; end;
         exit(R); end; }}"
    );
    al_testkit::run_in(&src, "C", "F")
}

fn text(bytes: &[u8], content_type: &str) -> String {
    match read(bytes, content_type) {
        Ok(Value::Text(s)) => s.to_string(),
        other => panic!("{other:?}"),
    }
}

const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

fn with_bom(rest: &[u8]) -> Vec<u8> {
    [&BOM[..], rest].concat()
}

#[test]
fn one_leading_utf8_bom_is_left_out_without_a_charset_or_with_utf8() {
    // Cells A_Utf8Bom_NoContentType, B_Utf8Bom_AppXml, C_Utf8Bom_AppXmlCharsetUtf8.
    for content_type in ["", "application/xml", "application/xml; charset=utf-8"] {
        assert_eq!(
            text(&with_bom(b"<a/>"), content_type),
            "4:60,97,47,62,",
            "{content_type}"
        );
    }
    // Cell D_Utf8BomOnly.
    assert_eq!(text(&BOM, ""), "0:");
    // Cell E_Utf8BomTwice: only the first mark goes.
    assert_eq!(text(&with_bom(&with_bom(b"a")), ""), "2:65279,97,");
}

#[test]
fn a_utf8_bom_after_the_start_or_under_latin1_stays() {
    // Cell F_Utf8BomInMiddle.
    assert_eq!(text(&[b'a', 0xEF, 0xBB, 0xBF, b'b'], ""), "3:97,65279,98,");
    // Cell G_Utf8Bom_CharsetLatin1: each byte is a character.
    assert_eq!(
        text(&with_bom(b"a"), "text/plain; charset=iso-8859-1"),
        "4:239,187,191,97,"
    );
}

#[test]
fn write_from_text_with_a_leading_feff_reads_without_it() {
    // Cell O_WriteFromText_WithLeadingFeff: the text is written as UTF-8, so the body starts with EF BB BF.
    let src = "codeunit 50100 C { procedure F(): Text var C: HttpContent; Ch: Char; T: Text; N: Integer;
        begin Ch := 65279; C.WriteFrom(Format(Ch) + 'a'); C.ReadAs(T); Ch := T[1]; N := Ch;
        exit(Format(StrLen(T)) + ':' + Format(N)); end; }";
    match al_testkit::run_in(src, "C", "F") {
        Ok(Value::Text(s)) => assert_eq!(s.to_string(), "1:97"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn unmeasured_byte_order_marks_are_refused() {
    // A UTF-16 mark under charset utf-16 and a UTF-8 mark read as a SecretText were not measured.
    match read(&[0xFF, 0xFE, b'a', 0], "text/plain; charset=utf-16") {
        Err(Signal::Unsupported(why)) => assert!(why.contains("byte order mark"), "{why}"),
        other => panic!("{other:?}"),
    }
    let src = "codeunit 50100 C { procedure F(): Boolean var C: HttpContent; Ch: Char; S: SecretText;
        begin Ch := 65279; C.WriteFrom(Format(Ch) + 'a'); exit(C.ReadAs(S)); end; }";
    match al_testkit::run_in(src, "C", "F") {
        Err(Signal::Unsupported(why)) => assert!(why.contains("byte order mark"), "{why}"),
        other => panic!("{other:?}"),
    }
}

//! `Factory.GetHeader()."Field" := value` assigns to a field of the record a procedure returns. That record is a copy, so
//! the assignment changes nothing the caller can read again (Continia's Delivery Network extended tests, "Validate eOrder
//! Header": `Factory.GetEOrderHeader()."Change Sequence Number" := ...`; 40 tests were refused with "is not assignable").
//! Not pinned by the corpus (docs/probes-needed.md).

use al_testkit::{run_in, text};

const SRC: &str = r#"
table 69830 "RR Header" { fields { field(1; "No."; Code[20]) { } field(2; Seq; Integer) { } } keys { key(PK; "No.") { } } }
codeunit 69831 "RR Factory"
{
    var Header: Record "RR Header";
    procedure GetHeader(): Record "RR Header" begin exit(Header); end;
    procedure Calls(): Integer begin exit(CallCount); end;
    procedure CountedHeader(): Record "RR Header" begin CallCount += 1; exit(Header); end;
    var CallCount: Integer;
}
codeunit 69832 "RR Caller"
{
    procedure F(): Text
    var Factory: Codeunit "RR Factory";
    begin
        Factory.GetHeader()."No." := 'CHANGED';
        Factory.CountedHeader().Seq := Factory.GetHeader().Seq + 1;
        exit(StrSubstNo('%1|%2|%3', Factory.GetHeader()."No.", Factory.GetHeader().Seq, Factory.Calls()));
    end;

    procedure TooLong()
    var Factory: Codeunit "RR Factory";
    begin
        Factory.GetHeader()."No." := PadStr('', 30, 'X');
    end;
}
"#;

#[test]
fn assigning_to_a_field_of_a_returned_record_runs_the_call_and_changes_no_record() {
    assert_eq!(text(run_in(SRC, "RR Caller", "F")), "|0|1");
}

#[test]
fn a_value_too_long_for_the_field_still_raises() {
    let err = run_in(SRC, "RR Caller", "TooLong").unwrap_err();
    assert!(format!("{err:?}").contains("length"), "{err:?}");
}

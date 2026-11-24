//! A table field whose CaptionClass expression reads the caption of that same field (al-runner issue 8329). Not pinned by the
//! corpus: every expected value is what BC 29.0.54011.55616 answered to the probe of assumption
//! `field-caption-inside-own-caption-class` (the probe test each case names), or the reporter's BC 29.1 result.
//! `Record.FieldCaption` of a field whose `Record.FieldCaption` is running gives the field's own caption; `FieldRef.Caption` and
//! a page control's caption run the expression without that guard, so a `FieldCaption` inside them runs it once more.

use al_builtins::native::Signal;
use al_builtins::value::Value;

/// The objects of the reporter's repro (al-runner issue 8329), with neighbors that tell the nested read's text apart: the
/// expression wraps it in `X`..`Y`, reads it through a codeunit on another record variable, gives it back without a comma,
/// reads another field whose expression reads its own caption (Document Capture's custom search fields), and two fields
/// read each other's. Another table holds a field whose expression reads its own caption by FieldRef.
const SRC: &str = r#"
table 50100 "ALR Caption Repro"
{
    fields
    {
        field(1; "Code"; Code[20]) { }
        field(2; "End Date"; Date)
        {
            Caption = 'End Date';
            CaptionClass = '3,' + FieldCaption("End Date");
        }
        field(3; "Other Ref"; Date) { Caption = 'Other Ref'; CaptionClass = '3,' + FieldCaption("Code"); }
        field(4; "Ping"; Date) { Caption = 'Ping'; CaptionClass = '3,P' + FieldCaption("Pong") + 'p'; }
        field(5; "Pong"; Date) { Caption = 'Pong'; CaptionClass = '3,Q' + FieldCaption("Ping") + 'q'; }
        field(6; "Wrapped"; Date) { Caption = 'Wrapped'; CaptionClass = '3,X' + FieldCaption("Wrapped") + 'Y'; }
        field(7; "No Caption Prop"; Date) { CaptionClass = '3,X' + FieldCaption("No Caption Prop") + 'Y'; }
        field(8; "Via"; Date) { Caption = 'Via'; CaptionClass = Mgt.Via(); }
        field(9; "No Comma"; Date) { Caption = 'No Comma'; CaptionClass = Mgt.NoComma(); }
        field(10; "Twin A"; Date) { Caption = 'Twin A'; CaptionClass = Mgt.Twin(10); }
        field(11; "Twin B"; Date) { Caption = 'Twin B'; CaptionClass = Mgt.Twin(11); }
    }
    keys { key(PK; "Code") { Clustered = true; } }

    var
        Mgt: Codeunit "ALR Caption Mgt";
}

table 50102 "ALR Ref Repro"
{
    fields
    {
        field(1; "Code"; Code[20]) { }
        field(2; "Ref Self"; Date) { Caption = 'Ref Self'; CaptionClass = '3,' + RefCaption(2); }
    }
    keys { key(PK; "Code") { Clustered = true; } }

    procedure RefCaption(No: Integer): Text
    var
        RecRef: RecordRef;
    begin
        RecRef.GetTable(Rec);
        exit(RecRef.Field(No).Caption);
    end;
}

codeunit 50103 "ALR Caption Mgt"
{
    procedure Via(): Text
    var
        Other: Record "ALR Caption Repro";
    begin
        exit('3,X' + Other.FieldCaption("Via") + 'Y');
    end;

    procedure NoComma(): Text
    var
        Other: Record "ALR Caption Repro";
    begin
        exit(Other.FieldCaption("No Comma"));
    end;

    procedure Twin(No: Integer): Text
    var
        Other: Record "ALR Caption Repro";
        Caption: Text;
    begin
        Caption := Other.FieldCaption("Twin A");
        if No = 11 then
            exit('3,B' + Caption);
        exit(Caption);
    end;
}

page 50100 "ALR Caption Repro Card"
{
    PageType = Card;
    SourceTable = "ALR Caption Repro";
    UsageCategory = None;
    layout
    {
        area(Content)
        {
            field("Code"; Rec."Code") { }
            field("End Date"; Rec."End Date") { }
            field("Wrapped"; Rec."Wrapped") { }
            field("Via"; Rec."Via") { }
            field("No Comma"; Rec."No Comma") { }
            field("Twin B"; Rec."Twin B") { }
        }
    }
}

codeunit 50101 C
{
    procedure SelfFieldCaption(): Text
    var
        Rec: Record "ALR Caption Repro";
    begin
        exit(Rec.FieldCaption("End Date"));
    end;

    procedure FieldCaptions(): Text
    var
        Rec: Record "ALR Caption Repro";
    begin
        exit(Rec.FieldCaption("Wrapped") + '|' + Rec.FieldCaption("No Caption Prop") + '|' + Rec.FieldCaption("Via") + '|' +
            Rec.FieldCaption("No Comma") + '|' + Rec.FieldCaption("Twin A") + '|' + Rec.FieldCaption("Twin B"));
    end;

    procedure SelfTestPageCaption(): Text
    var
        Card: TestPage "ALR Caption Repro Card";
    begin
        Card.OpenNew();
        exit(Card."End Date".Caption);
    end;

    procedure TestPageCaptions(): Text
    var
        Card: TestPage "ALR Caption Repro Card";
    begin
        Card.OpenView();
        exit(Card."Wrapped".Caption + '|' + Card."Via".Caption + '|' + Card."No Comma".Caption + '|' + Card."Twin B".Caption);
    end;

    procedure FieldRefCaptions(): Text
    var
        RecRef: RecordRef;
    begin
        RecRef.Open(Database::"ALR Caption Repro");
        exit(RecRef.Field(2).Caption + '|' + RecRef.Field(6).Caption + '|' + RecRef.Field(8).Caption + '|' +
            RecRef.Field(9).Caption + '|' + RecRef.Field(11).Caption);
    end;

    procedure NestedFieldRefCaption(): Text
    var
        Rec: Record "ALR Ref Repro";
    begin
        exit(Rec.FieldCaption("Ref Self"));
    end;

    procedure MutualFieldCaption(): Text
    var
        Rec: Record "ALR Caption Repro";
    begin
        exit(Rec.FieldCaption("Ping") + '|' + Rec.FieldCaption("Pong"));
    end;

    procedure OtherFieldCaption(): Text
    var
        Rec: Record "ALR Caption Repro";
    begin
        exit(Rec.FieldCaption("Other Ref"));
    end;
}
"#;

fn out(routine: &str) -> String {
    match al_testkit::Built::from_source(SRC).run_in("C", routine) {
        Ok(Value::Text(t)) => t.to_string(),
        Err(Signal::Unsupported(why)) => format!("refused:{why}"),
        other => format!("{other:?}"),
    }
}

/// The reporter's repro (BC 29.1): `'3,' + FieldCaption(<same field>)` reads as the field's caption.
#[test]
fn field_caption_inside_its_own_caption_class_gives_the_caption() {
    assert_eq!(out("SelfFieldCaption"), "End Date");
}

/// The nested `FieldCaption` gives the field's caption (its name without a Caption), on the record or on another record
/// variable through a codeunit, and a text without a comma stays as it is. Probe T01, T06, T02, T03, P03 (Twin A, Twin B).
#[test]
fn a_nested_field_caption_of_the_same_field_gives_its_caption() {
    assert_eq!(
        out("FieldCaptions"),
        "XWrappedY|XNo Caption PropY|XViaY|No Comma|Twin A|BTwin A"
    );
}

/// The reporter's repro (BC 29.1): a TestPage control shows the caption.
#[test]
fn testpage_caption_of_a_self_referencing_caption_class_gives_the_caption() {
    assert_eq!(out("SelfTestPageCaption"), "End Date");
}

/// A control's caption runs the expression without the guard, so the `FieldCaption` inside runs it once more: probe P01
/// (`[XXSelf DirectYY] [XXVia PureYY] [No Comma Pure] [Twin A] [BTwin A]`).
#[test]
fn a_page_control_caption_runs_the_expression_once_more() {
    assert_eq!(out("TestPageCaptions"), "XXWrappedYY|XXViaYY|No Comma|BTwin A");
}

/// `FieldRef.Caption` runs the expression without the guard too: probe T07 (`XXVia CodeunitYY`), T15 (`XXSelf DirectYY`,
/// `No Comma Pure`), P04 (`BTwin A`).
#[test]
fn field_ref_caption_runs_the_expression_once_more() {
    assert_eq!(out("FieldRefCaptions"), "End Date|XXWrappedYY|XXViaYY|No Comma|BTwin A");
}

/// A FieldRef.Caption of the same field inside its own expression recurses until BC runs out of memory (probe T14): refused.
#[test]
fn a_nested_field_ref_caption_of_the_same_field_refuses() {
    let got = out("NestedFieldRefCaption");
    assert!(
        got.starts_with("refused:") && got.contains("its own CaptionClass expression runs"),
        "{got}"
    );
}

/// Probe T13: `[PQPingqp] [QPPongpq]`.
#[test]
fn caption_classes_that_read_each_other_give_the_inner_caption() {
    assert_eq!(out("MutualFieldCaption"), "PQPingqp|QPPongpq");
}

/// Main's result is kept: an expression that reads another field's caption runs that read as before.
#[test]
fn caption_class_reading_another_field_keeps_mains_result() {
    assert_eq!(out("OtherFieldCaption"), "Code");
}

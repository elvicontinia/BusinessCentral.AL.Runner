//! Rec.SetRecFilter() in the OnOpenPage of a page an action opens through RunPageLink (run_page_link_open.rs), then the page.
//! Not pinned by the corpus: every expected value is what BC 29.0.54011.55616 answered to the probe of assumption
//! `run-page-link-open-set-rec-filter` (each test names its cell). Rec holds the link as a filter with the linked fields filled;
//! SetRecFilter filters the primary key on the buffer's values and keeps the link's filter on another field.

const SRC: &str = r#"
table 50990 "ALR RPL Probe"
{
    fields { field(1; "No."; Code[20]) { } field(2; Name; Text[30]) { } field(3; Grp; Integer) { } }
    keys { key(PK; "No.") { Clustered = true; } key(ByGrp; Grp) { } }
}
codeunit 50992 "ALR RPL Log"
{
    SingleInstance = true;
    var Lines: Text;
    procedure Add(Line: Text) begin Lines += Line; end;
    procedure Take() Result: Text begin Result := Lines; Lines := ''; end;
}
page 50993 "ALR RPL Card"
{
    PageType = Card; SourceTable = "ALR RPL Probe"; UsageCategory = None;
    layout { area(Content) { field("No."; Rec."No.") { } field(Name; Rec.Name) { } field(Grp; Rec.Grp) { } } }
    trigger OnOpenPage()
    begin
        Log.Add(StrSubstNo('pre[no=%1 name=%2 grp=%3 gf=%4 gfs=%5]', Rec."No.", Rec.Name, Rec.Grp, Rec.GetFilter("No."), Rec.GetFilters()));
        Rec.SetRecFilter();
        Log.Add(StrSubstNo(' post[gf=%1 gfgrp=%2 gfs=%3 count=%4]', Rec.GetFilter("No."), Rec.GetFilter(Grp), Rec.GetFilters(), Rec.Count()));
    end;
    var Log: Codeunit "ALR RPL Log";
}
page 50994 "ALR RPL List"
{
    PageType = List; SourceTable = "ALR RPL Probe"; UsageCategory = None;
    layout { area(Content) { repeater(Lines) { field("No."; Rec."No.") { } field(Name; Rec.Name) { } field(Grp; Rec.Grp) { } } } }
    trigger OnOpenPage()
    begin
        Log.Add(StrSubstNo('pre[no=%1 name=%2 grp=%3 gf=%4 gfs=%5]', Rec."No.", Rec.Name, Rec.Grp, Rec.GetFilter("No."), Rec.GetFilters()));
        Rec.SetRecFilter();
        Log.Add(StrSubstNo(' post[gf=%1 gfgrp=%2 gfs=%3 count=%4]', Rec.GetFilter("No."), Rec.GetFilter(Grp), Rec.GetFilters(), Rec.Count()));
    end;
    var Log: Codeunit "ALR RPL Log";
}
page 50995 "ALR RPL Host"
{
    PageType = List; SourceTable = "ALR RPL Probe"; UsageCategory = None;
    layout { area(Content) { repeater(Lines) { field("No."; Rec."No.") { } field(Name; Rec.Name) { } field(Grp; Rec.Grp) { } } } }
    actions
    {
        area(Processing)
        {
            action(CardPK) { RunObject = page "ALR RPL Card"; RunPageLink = "No." = field("No."); RunPageView = sorting("No."); Scope = Repeater; }
            action(CardPKNoView) { RunObject = page "ALR RPL Card"; RunPageLink = "No." = field("No."); Scope = Repeater; }
            action(CardGrp) { RunObject = page "ALR RPL Card"; RunPageLink = Grp = field(Grp); Scope = Repeater; }
            action(ListPK) { RunObject = page "ALR RPL List"; RunPageLink = "No." = field("No."); Scope = Repeater; }
        }
    }
}
table 50991 "ALR RPL Two"
{
    fields { field(1; Grp; Integer) { } field(2; "No."; Code[20]) { } }
    keys { key(PK; Grp, "No.") { Clustered = true; } }
}
page 50996 "ALR RPL Two Card"
{
    PageType = Card; SourceTable = "ALR RPL Two"; UsageCategory = None;
    layout { area(Content) { field("No."; Rec."No.") { } } }
    trigger OnOpenPage() begin Rec.SetRecFilter(); end;
}
page 50998 "ALR RPL Two Host"
{
    PageType = List; SourceTable = "ALR RPL Two"; UsageCategory = None;
    layout { area(Content) { repeater(Lines) { field("No."; Rec."No.") { } } } }
    actions { area(Processing) { action(OnGrp) { RunObject = page "ALR RPL Two Card"; RunPageLink = Grp = field(Grp); Scope = Repeater; } } }
}
codeunit 50997 C
{
    var Log: Codeunit "ALR RPL Log";

    procedure TwoFieldKey(): Text
    var Rec: Record "ALR RPL Two"; Host: TestPage "ALR RPL Two Host"; Card: TestPage "ALR RPL Two Card";
    begin
        Rec.Grp := 1; Rec."No." := 'B1'; Rec.Insert();
        Host.OpenView(); Host.First(); Card.Trap(); Host.OnGrp.Invoke();
        exit(Card."No.".Value);
    end;

    procedure CardPK(): Text
    var Host: TestPage "ALR RPL Host"; Card: TestPage "ALR RPL Card";
    begin
        Setup(Host); Card.Trap(); Host.CardPK.Invoke();
        exit(Log.Take() + ' ' + DescribeCard(Card));
    end;

    procedure CardPKNoView(): Text
    var Host: TestPage "ALR RPL Host"; Card: TestPage "ALR RPL Card";
    begin
        Setup(Host); Card.Trap(); Host.CardPKNoView.Invoke();
        exit(Log.Take() + ' ' + DescribeCard(Card));
    end;

    procedure CardGrp(): Text
    var Host: TestPage "ALR RPL Host"; Card: TestPage "ALR RPL Card";
    begin
        Setup(Host); Card.Trap(); Host.CardGrp.Invoke();
        exit(Log.Take() + ' ' + DescribeCard(Card));
    end;

    procedure ListPK(): Text
    var Host: TestPage "ALR RPL Host"; List: TestPage "ALR RPL List"; Rows: Text;
    begin
        Setup(Host); List.Trap(); Host.ListPK.Invoke();
        if List.First() then
            repeat Rows += List."No.".Value + ','; until not List.Next();
        exit(Log.Take() + ' page[rows=' + Rows + ']');
    end;

    local procedure DescribeCard(var Card: TestPage "ALR RPL Card"): Text
    var First: Text; Moved: Boolean;
    begin
        First := Card."No.".Value + '/' + Card.Name.Value;
        Moved := Card.Next();
        exit(StrSubstNo('page[first=%1 next=%2 then=%3]', First, Moved, Card."No.".Value));
    end;

    local procedure Setup(var Host: TestPage "ALR RPL Host")
    var Rec: Record "ALR RPL Probe";
    begin
        Ins(Rec, 'A1', 'One', 1); Ins(Rec, 'A2', 'Two', 1); Ins(Rec, 'A3', 'Three', 2);
        Host.OpenView(); Host.GoToKey('A2'); Log.Take();
    end;

    local procedure Ins(var Rec: Record "ALR RPL Probe"; No: Code[20]; Name: Text[30]; Grp: Integer)
    begin
        Rec.Init(); Rec."No." := No; Rec.Name := Name; Rec.Grp := Grp; Rec.Insert();
    end;
}
"#;

fn run(proc_name: &str) -> String {
    let mut built = al_testkit::Built::from_source(SRC);
    al_testkit::text(built.run_in("C", proc_name))
}

const ON_KEY: &str = "pre[no=A2 name= grp=0 gf=A2 gfs=No.: A2] post[gf=A2 gfgrp= gfs=No.: A2 count=1]";

/// Cell R1: a Card linked on its key with a RunPageView sorting (Document Capture's Document Card).
#[test]
fn set_rec_filter_on_a_card_linked_on_its_key_with_a_run_page_view() {
    assert_eq!(run("CardPK"), format!("{ON_KEY} page[first=A2/Two next=No then=A2]"));
}

/// Cell R2: the same without a RunPageView.
#[test]
fn set_rec_filter_on_a_card_linked_on_its_key() {
    assert_eq!(
        run("CardPKNoView"),
        format!("{ON_KEY} page[first=A2/Two next=No then=A2]")
    );
}

/// Cell R3: linked on another field the buffer's key is blank, so SetRecFilter filters `No.: ''` beside the link and the page
/// shows no row.
#[test]
fn set_rec_filter_on_a_card_linked_on_another_field() {
    assert_eq!(
        run("CardGrp"),
        "pre[no= name= grp=1 gf= gfs=Grp: 1] post[gf='' gfgrp=1 gfs=No.: '', Grp: 1 count=0] page[first=/ next=No then=]"
    );
}

/// Cell R4: a List target linked on its key.
#[test]
fn set_rec_filter_on_a_list_linked_on_its_key() {
    assert_eq!(run("ListPK"), format!("{ON_KEY} page[rows=A2,A2,]"));
}

/// A key of several fields was not measured: SetRecFilter there keeps the refusal.
#[test]
fn set_rec_filter_with_a_key_of_several_fields_refuses() {
    let mut built = al_testkit::Built::from_source(SRC);
    match built.run_in("C", "TwoFieldKey") {
        Err(al_builtins::native::Signal::Unsupported(why)) => assert!(why.contains("Rec.SetRecFilter()"), "{why}"),
        other => panic!("{other:?}"),
    }
}

RunPageLinkOpenSetRecFilter => {
    id: "run-page-link-open-set-rec-filter",
    assumed: "Rec.SetRecFilter() in the OnOpenPage of a page an action opens through RunPageLink, in filter group 0 with the primary key as the current key, filters the primary key on Rec's buffer, whose linked fields are filled and the others blank, replaces the link's filter on a key field and keeps it on another field; Count, GetFilter and GetFilters then read those filters and the page opens on them. Seen on BC 29.0.54011.55616 with a partner probe: a Card linked on its key with and without RunPageView = sorting(<key>) reads `No.: A2` before and after, Count 1, and opens on A2 with no next row; a List linked on its key the same; a Card linked on another field reads `Grp: 1` before and `No.: '', Grp: 1` after, Count 0, and opens on no row. Another filter group, another current key and a key of several fields were not measured: refused.",
    tests: "Continia Document Capture Test Suite (CDC Order Change Valid. (Test), CDC Order Resp. Valid. (Test) and others): the Document Card action of CDC Document List With Image opens CDC Document Card through RunPageLink = \"No.\" = field(\"No.\") with RunPageView = sorting(\"No.\"), and its OnOpenPage calls Rec.SetRecFilter() and then Document.Get(Rec.GetFilter(\"No.\")).",
    probe: "A table with a one-field key and a field of a second key; a host list with actions that open a Card through RunPageLink on the key with and without RunPageView = sorting(<key>), a Card through RunPageLink on the other field, and a List through RunPageLink on the key; each target's OnOpenPage logs Rec's linked and other fields, GetFilter and GetFilters, calls Rec.SetRecFilter(), and logs GetFilter of both fields, GetFilters and Count; the test then reads the page's row and Next. Measured as a partner probe; the corpus probe (the file below) is still to be written. Still wanted: a key of several fields with the link on part of it, another filter group, a RunPageView with another sorting, BC 27 and 28.",
    awaiting: Awaiting {
        issues: &[],
        files: &["tests/al-language/testpage/RunPageLinkOpenSetRecFilter.al"],
        since: "2026-11-23",
    },
}

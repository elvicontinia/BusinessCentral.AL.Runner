AssignFieldOfReturnedRecord => {
    id: "assign-field-of-returned-record",
    assumed: "`Proc().Field := Value` on a Record a procedure returns runs the call, converts the value to the field (a value too long raises) and changes no record the caller can read: the returned record is a copy.",
    tests: "Continia Delivery Network extended tests, Validate eOrder Header: Factory.GetEOrderHeader().\"Change Sequence Number\" := ...",
    probe: "A codeunit with a global record and GetRec(): Record returning it; GetRec().\"No.\" := 'X'; then read GetRec().\"No.\"; also with a temporary record and with a value longer than the field.",
}

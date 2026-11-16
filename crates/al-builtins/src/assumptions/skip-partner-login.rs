SkipPartnerLogin => {
    id: "skip-partner-login",
    assumed: "With --skip-partner-login the OnAfterLogin subscribers of partner dependency apps do not run in the test session, although BC runs them there (PlLoginSingleTests.al): the test is assumed not to depend on what they leave behind. Opt-in only; a run without the flag runs and watches them.",
    tests: "Continia Delivery Network Extended Tests: Continia Core's login subscribers (src/Specialization/CompanyTriggersSubscr.Codeunit.al, src/Source/Core/Core.Codeunit.al ScheduleOnlineCommAfterLogInStart) fill caches of System Application's Environment Information Impl. and Server Setting Impl. and of Continia's CSC Permission Mgt.; every DN test codeunit's static reach is partly unknown, so the watch refuses all of them, while the suite passes on BC.",
    probe: "None planned: the flag is a deliberate deviation for partner suites. A run with and without it on a partner suite whose login subscribers are measured shows what differs.",
}

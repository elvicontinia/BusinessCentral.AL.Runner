//! `--allow-network`: a request that no `[HttpClientHandler]` answers goes to the host's `Network`, and its answer is
//! the response. Without a network the request stays refused (ADR-027).

use al_builtins::native::Signal;
use al_builtins::value::Value;
use al_host::network::{Network, NetworkRequest, NetworkResponse};
use al_testkit::text;
use std::sync::{Arc, Mutex};

const SRC: &str = r#"
codeunit 50100 C
{
    procedure Post(): Text
    var
        Client: HttpClient;
        Content: HttpContent;
        ContentHeaders: HttpHeaders;
        Response: HttpResponseMessage;
        ResponseContent: HttpContent;
        Token: SecretText;
        Body: Text;
        Types: List of [Text];
        Ids: List of [Text];
        Sent: Boolean;
    begin
        Token := 'Bearer abc';
        Client.DefaultRequestHeaders().Add('Accept', 'application/xml');
        Client.DefaultRequestHeaders().Add('Authorization', Token);
        Content.WriteFrom('<a/>');
        Content.GetHeaders(ContentHeaders);
        ContentHeaders.Remove('Content-Type');
        ContentHeaders.Add('Content-Type', 'application/xml');
        Sent := Client.Post('https://example.test/api/x?y=1', Content, Response);
        ResponseContent := Response.Content();
        ResponseContent.ReadAs(Body);
        ResponseContent.GetHeaders(ContentHeaders);
        ContentHeaders.GetValues('Content-Type', Types);
        Response.Headers().GetValues('X-Id', Ids);
        exit(Format(Sent) + '|' + Format(Response.HttpStatusCode()) + '|' + Format(Response.IsSuccessStatusCode()) + '|' +
            Response.ReasonPhrase() + '|' + Body + '|' + Types.Get(1) + '|' + Ids.Get(1));
    end;

    procedure GetFails(): Text
    var
        Client: HttpClient;
        Response: HttpResponseMessage;
    begin
        if Client.Get('https://example.test/down', Response) then
            exit('sent');
        exit(GetLastErrorText() + '|' + GetLastErrorCode() + '|' + Format(Response.HttpStatusCode()));
    end;
}
"#;

#[derive(Default)]
struct Recorded {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

struct FakeNetwork {
    answer: Result<(u16, &'static str), &'static str>,
    sent: Mutex<Vec<Recorded>>,
}

impl Network for FakeNetwork {
    fn send(&self, r: &NetworkRequest) -> Result<NetworkResponse, String> {
        self.sent.lock().unwrap().push(Recorded {
            method: r.method.clone(),
            url: r.url.clone(),
            headers: r.headers.clone(),
            body: r.body.clone(),
        });
        let (status, reason) = self.answer.map_err(String::from)?;
        Ok(NetworkResponse {
            status,
            reason: reason.into(),
            headers: vec![
                ("Content-Type".into(), "application/xml; charset=utf-8".into()),
                ("X-Id".into(), "42".into()),
            ],
            body: b"<ok/>".to_vec(),
        })
    }
}

fn run_on(network: Option<Arc<FakeNetwork>>, routine: &str) -> Result<Value, Signal> {
    let lowered = al_testkit::compile(SRC);
    let engine = al_testkit::engine(&lowered);
    let host = match network {
        Some(n) => al_host::Host::default().with_network(n),
        None => al_host::Host::default(),
    };
    let mut s = al_vm::Session::with_host(engine, host);
    let (oi, ri) = al_testkit::Built::new(lowered).routine(routine);
    let inst = s.instance(oi);
    s.call(oi, ri, inst, vec![]).map(|(v, _)| v)
}

#[test]
fn a_request_no_handler_answers_goes_to_the_network_with_its_headers_and_body() {
    let net = Arc::new(FakeNetwork {
        answer: Ok((201, "Created")),
        sent: Mutex::default(),
    });
    let r = run_on(Some(net.clone()), "Post");
    assert_eq!(text(r), "Yes|201|Yes|Created|<ok/>|application/xml; charset=utf-8|42");
    let sent = net.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    let req = &sent[0];
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://example.test/api/x?y=1");
    assert_eq!(req.body.as_deref(), Some(&b"<a/>"[..]));
    let header = |n: &str| {
        req.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(n))
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(header("Accept"), Some("application/xml"));
    assert_eq!(header("Authorization"), Some("Bearer abc"));
    assert_eq!(header("Content-Type"), Some("application/xml"));
}

#[test]
fn a_request_without_an_answer_fails_as_bc_fails_a_connection() {
    let net = Arc::new(FakeNetwork {
        answer: Err("connection refused"),
        sent: Mutex::default(),
    });
    let r = run_on(Some(net), "GetFails");
    assert_eq!(
        text(r),
        "There was an error while executing the HTTP request. Make sure you are connecting to a valid endpoint.|HttpRequest:Socket|0"
    );
}

#[test]
fn without_a_network_the_request_stays_refused() {
    match run_on(None, "GetFails") {
        Err(Signal::Unsupported(why)) => assert!(why.contains("never opens a connection"), "{why}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

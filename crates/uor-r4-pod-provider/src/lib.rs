#![forbid(unsafe_code)]
//! The admitted runner owns resource admission, create reconciliation and holds.
//! This adapter sends one mutation request per invocation, with no retries.
use serde_json::{json, Map, Value};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub type Result<T> = std::result::Result<T, String>;
const ORIGIN: &str = "https://console.vast.ai";
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub method: &'static str,
    pub path: String,
    pub query: Vec<(String, Value)>,
    pub body: Option<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Instances,
    Instance,
    Offers,
    Create,
    Destroy,
}

fn request(method: &'static str, path: String, body: Option<Value>) -> Request {
    Request {
        method,
        path,
        query: vec![],
        body,
    }
}
fn id(s: &str) -> Result<u64> {
    s.parse::<u64>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "expected a positive numeric offer/instance ID".into())
}
fn positive(s: &str) -> Result<f64> {
    s.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n > 0.)
        .ok_or_else(|| "expected a finite positive storage amount".into())
}

/// Shell-style token splitting only; nothing is executed or interpolated.
fn words(s: &str) -> Result<Vec<String>> {
    let mut out = vec![];
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;
    for c in s.chars() {
        if escaped {
            word.push(c);
            escaped = false;
            started = true;
            continue;
        }
        if c == '\\' && quote != Some('\'') {
            escaped = true;
            started = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None
            } else {
                word.push(c)
            };
            started = true;
        } else if c == '\'' || c == '"' {
            quote = Some(c);
            started = true;
        } else if c.is_whitespace() {
            if started {
                out.push(std::mem::take(&mut word));
                started = false;
            }
        } else {
            word.push(c);
            started = true;
        }
    }
    if escaped || quote.is_some() {
        return Err("unterminated quoting in provider argument".into());
    }
    if started {
        out.push(word)
    }
    Ok(out)
}

fn env_map(s: &str) -> Result<Value> {
    let w = words(s)?;
    let mut env = Map::new();
    let mut i = 0;
    while i < w.len() {
        let flag = w[i].as_str();
        let value = w.get(i + 1).ok_or("environment option lacks a value")?;
        match flag {
            "-e" => {
                let (name, val) = value
                    .split_once('=')
                    .ok_or("environment assignment requires NAME=VALUE")?;
                if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Err("invalid environment variable name".into());
                }
                env.insert(name.into(), Value::String(val.into()));
            }
            "-p" => {
                if value.is_empty() || !value.chars().all(|c| "0123456789:tcp/udp".contains(c)) {
                    return Err("invalid port mapping".into());
                }
                env.insert(format!("-p {value}"), json!("1"));
            }
            "-h" => {
                env.insert("-h".into(), json!(value));
            }
            "-v" | "-n" => {
                if value.is_empty()
                    || !value
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || ":./_-".contains(c))
                {
                    return Err("invalid container option".into());
                }
                env.insert(format!("{flag} {value}"), json!("1"));
            }
            _ => {
                return Err(
                    "unsupported environment option; use explicit -e/-p/-h/-v/-n syntax".into(),
                )
            }
        }
        i += 2;
    }
    Ok(Value::Object(env))
}

fn alias(field: &str) -> &str {
    match field {
        "cuda_vers" => "cuda_max_good",
        "dph" => "dph_total",
        other => other,
    }
}
fn offer_query(s: &str) -> Result<Value> {
    let mut q = json!({"verified":{"eq":true},"external":{"eq":false},"rentable":{"eq":true}});
    for term in words(s)? {
        let (at, op) = term
            .char_indices()
            .find(|(_, c)| "=<>!".contains(*c))
            .map(|(i, _)| {
                let rest = &term[i..];
                let op = [">=", "<=", "!=", "==", "=", ">", "<"]
                    .into_iter()
                    .find(|o| rest.starts_with(o))
                    .unwrap_or("");
                (i, op)
            })
            .ok_or("unsupported offer query; use FIELD=VALUE or FIELD>=VALUE")?;
        let field = &term[..at];
        let value = &term[at + op.len()..];
        if ![
            "gpu_name",
            "num_gpus",
            "datacenter",
            "verified",
            "rentable",
            "external",
            "rented",
            "reliability",
            "cuda_vers",
            "cuda_max_good",
            "disk_space",
            "inet_down",
            "inet_up",
            "gpu_ram",
            "gpu_total_ram",
            "cpu_ram",
            "dph",
            "dph_total",
        ]
        .contains(&field)
            || value.is_empty()
            || op.is_empty()
        {
            return Err("unsupported offer query field or missing value".into());
        }
        let op = match op {
            "=" | "==" => "eq",
            ">=" => "gte",
            "<=" => "lte",
            ">" => "gt",
            "<" => "lt",
            "!=" => "neq",
            _ => return Err("unsupported query operator".into()),
        };
        let field = alias(field);
        let value = if ["gpu_ram", "gpu_total_ram", "cpu_ram"].contains(&field) {
            let n = value.parse::<f64>().map_err(|_| "invalid RAM filter")? * 1000.;
            if !n.is_finite() {
                return Err("nonfinite RAM filter".into());
            }
            json!(n)
        } else {
            match value {
                "true" | "True" => json!(true),
                "false" | "False" => json!(false),
                "null" | "None" => Value::Null,
                _ => json!(value.replace('_', " ")),
            }
        };
        if q.get(field).is_none() {
            q[field] = json!({});
        }
        q[field][op] = value;
    }
    Ok(q)
}

/// Build exact REST requests for the five admitted Vast command shapes.
/// Unknown flags refuse before credentials or network are touched.
pub fn parse(args: &[String]) -> Result<(Operation, Request)> {
    let args = if args.first().is_some_and(|a| a == "vast") {
        &args[1..]
    } else {
        args
    };
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--raw")
        .collect();
    match args.as_slice() {
        ["show","instances"] => {
            let mut r=request("GET","/api/v1/instances/".into(),None);
            r.query=vec![("select_filters".into(),json!({})),("order_by".into(),json!([{"col":"id","dir":"asc"}])),("limit".into(),json!(25))];
            Ok((Operation::Instances,r))
        }
        ["show","instance",n] => {let mut r=request("GET",format!("/api/v0/instances/{}/",id(n)?),None);r.query.push(("owner".into(),json!("me")));Ok((Operation::Instance,r))}
        ["destroy","instance",n] => Ok((Operation::Destroy,request("DELETE",format!("/api/v0/instances/{}/",id(n)?),Some(json!({}))))),
        ["search","offers",query,flags @ ..] => {
            let mut q=offer_query(query)?; let mut storage=5.; let mut order="score-"; let mut limit=None; let mut i=0;
            while i<flags.len() {
                let value=flags.get(i+1).ok_or("offer flag lacks a value")?;
                match flags[i] {"--storage"=>storage=positive(value)?,"-o"|"--order"=>order=value,"--limit"=>limit=Some(id(value)?),_=>return Err("unsupported offer-search flag".into())} i+=2;
            }
            let mut ordering=vec![];
            for item in order.split(',') {
                if item.is_empty() {continue}
                let descending=item.starts_with('-') || item.ends_with('-');
                let field=alias(item.trim_matches(['+','-']));
                if !["score","dph_total","reliability","reliability2","gpu_name","num_gpus"].contains(&field) {return Err("unsupported offer order field".into())}
                ordering.push(json!([field,if descending {"desc"} else {"asc"}]));
            }
            q["order"]=json!(ordering);q["type"]=json!("on-demand");q["allocated_storage"]=json!(storage);
            if let Some(limit)=limit {q["limit"]=json!(limit)}
            Ok((Operation::Offers,request("POST","/api/v0/bundles/".into(),Some(q))))
        }
        ["create","instance",offer,flags @ ..] => {
            let offer=id(offer)?; let mut image=None; let mut label=None; let mut disk=10.;let mut env=json!({});let mut onstart=None;let mut ssh=false;let mut direct=false;let mut i=0;
            while i<flags.len() {
                match flags[i] {
                    "--ssh"=>ssh=true,"--direct"=>direct=true,
                    "--image"|"--label"|"--disk"|"--env"|"--onstart-cmd"=>{
                        let val=*flags.get(i+1).ok_or("create flag lacks a value")?;
                        match flags[i] {"--image"=>image=Some(val),"--label"=>label=Some(val),"--disk"=>disk=positive(val)?,"--env"=>env=env_map(val)?,"--onstart-cmd"=>onstart=Some(val),_=>{}}
                        i+=1;
                    }
                    _=>return Err("unsupported instance-create flag".into()),
                } i+=1;
            }
            let image=image.filter(|s|!s.is_empty()).ok_or("create requires --image")?;
            let label=label.filter(|s|!s.is_empty()).ok_or("create requires --label for duplicate-rental reconciliation")?;
            let runtype=if ssh && direct {"ssh_direc ssh_proxy"} else if ssh {"ssh_proxy"} else {"ssh"};
            let body=json!({"client_id":"me","image":image,"env":env,"price":null,"disk":disk,"label":label,"extra":null,"onstart":onstart,"image_login":null,"python_utf8":false,"lang_utf8":false,"use_jupyter_lab":false,"jupyter_dir":null,"force":false,"cancel_unavail":false,"template_hash_id":null,"user":null,"runtype":runtype});
            Ok((Operation::Create,request("PUT",format!("/api/v0/asks/{offer}/"),Some(body))))
        }
        _=>Err("supported commands: show instances; show instance ID; search offers QUERY; create instance OFFER; destroy instance ID".into()),
    }
}

fn credentials() -> Result<String> {
    let key = match std::env::var("VAST_API_KEY") {
        Ok(key) => key,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err("VAST_API_KEY is not valid UTF-8".into())
        }
        Err(std::env::VarError::NotPresent) => {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or("HOME is unavailable for Vast credential lookup")?;
            let config = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            let paths = [
                config.join("vastai/vast_api_key"),
                home.join(".vast_api_key"),
            ];
            let mut found = None;
            for path in paths {
                match fs::read_to_string(path) {
                    Ok(k) => {
                        found = Some(k);
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err("cannot read configured Vast credential file".into()),
                }
            }
            found.ok_or("Vast authentication unavailable: set VAST_API_KEY or configure the private Vast key file")?
        }
    };
    let key = key.trim().to_owned();
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_graphic() && !b"\"\\".contains(&b))
    {
        return Err("Vast credential has an invalid header representation".into());
    }
    Ok(key)
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char)
        } else {
            out.push_str(&format!("%{b:02X}"))
        }
    }
    out
}
fn url(r: &Request) -> String {
    let mut url = format!("{ORIGIN}{}", r.path);
    for (i, (k, v)) in r.query.iter().enumerate() {
        url.push(if i == 0 { '?' } else { '&' });
        url.push_str(k);
        url.push('=');
        url.push_str(&encode(
            &v.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| v.to_string()),
        ));
    }
    url
}
fn quoted(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}
fn curl_config(r: &Request, key: &str) -> String {
    let mut s = format!(
        "url = {}\nrequest = {}\nheader = {}\nheader = \"Content-Type: application/json\"\n",
        quoted(&url(r)),
        quoted(r.method),
        quoted(&format!("Authorization: Bearer {key}"))
    );
    if let Some(body) = &r.body {
        s.push_str(&format!("data = {}\n", quoted(&body.to_string())))
    }
    s
}

fn send(r: &Request, key: &str) -> Result<Value> {
    // stdin-only config prevents credential/body disclosure through process argv.
    // --disable must be first: no user curlrc may enable retries, redirects or logs.
    let mut child = Command::new("curl")
        .args([
            "--disable",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--connect-timeout",
            "20",
            "--max-time",
            "120",
            "--max-redirs",
            "0",
            "--retry",
            "0",
            "--write-out",
            "\n%{http_code}",
            "--config",
            "-",
        ])
        .env_remove("VAST_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "curl TLS transport could not start")?;
    let write = child
        .stdin
        .take()
        .ok_or("curl stdin unavailable")
        .and_then(|mut stdin| {
            stdin
                .write_all(curl_config(r, key).as_bytes())
                .map_err(|_| "curl request configuration could not be sent")
        });
    if let Err(error) = write {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.into());
    }
    let out = child
        .wait_with_output()
        .map_err(|_| "curl transport wait failed")?;
    if !out.status.success() {
        return Err(format!("Vast transport failed (curl exit {}); mutation was not retried; reconcile instance label before retrying create",out.status.code().unwrap_or(-1)));
    }
    if out.stdout.len() > MAX_RESPONSE_BYTES {
        return Err("Vast response exceeded the admitted response bound".into());
    }
    let text = String::from_utf8(out.stdout).map_err(|_| "Vast returned non-UTF8 data")?;
    let (body, status) = text
        .rsplit_once('\n')
        .ok_or("Vast transport omitted HTTP status")?;
    let code = status
        .parse::<u16>()
        .map_err(|_| "Vast returned an invalid HTTP status")?;
    if !(200..300).contains(&code) {
        return Err(format!(
            "Vast HTTP {code}; {}",
            match code {
                401 | 403 => "authentication/authorization refused",
                429 => "rate limited; no mutation retry",
                500..=599 => "provider failure; reconcile create label before retry",
                _ => "request refused (provider body suppressed to protect credentials)",
            }
        ));
    }
    serde_json::from_str(body).map_err(|_| {
        "Vast returned invalid JSON; reconcile instance label before retrying create".into()
    })
}

const INSTANCE_FIELDS: &[&str] = &[
    "id",
    "label",
    "intended_status",
    "cur_state",
    "num_gpus",
    "dph_total",
    "start_date",
    "actual_status",
    "geolocation",
    "gpu_name",
    "public_ipaddr",
    "ports",
    "ssh_host",
    "ssh_port",
];
const OFFER_FIELDS: &[&str] = &[
    "id",
    "dph_total",
    "num_gpus",
    "geolocation",
    "reliability2",
    "gpu_name",
    "cuda_max_good",
    "disk_space",
    "inet_down",
    "datacenter",
    "verified",
    "rentable",
];
fn project(v: &Value, fields: &[&str]) -> Result<Value> {
    let object = v
        .as_object()
        .ok_or("Vast inventory contains a non-object row")?;
    if object.get("id").and_then(Value::as_u64).is_none() {
        return Err("Vast inventory row lacks a numeric identity".into());
    }
    if fields == INSTANCE_FIELDS {
        if object.get("num_gpus").and_then(Value::as_u64).is_none() {
            return Err(
                "Vast instance lacks a nonnegative integer GPU count; admission inventory refused"
                    .into(),
            );
        }
        if !object
            .get("dph_total")
            .and_then(Value::as_f64)
            .is_some_and(|n| n.is_finite() && n >= 0.)
        {
            return Err("Vast instance lacks a finite nonnegative hourly price; admission inventory refused".into());
        }
        // Unknown/missing statuses are conservatively counted as RUNNING by
        // the admitted runner. Do not synthesize a stopped state here.
    }
    Ok(Value::Object(
        fields
            .iter()
            .filter_map(|key| object.get(*key).map(|v| ((*key).into(), v.clone())))
            .collect(),
    ))
}

pub fn normalize(operation: &Operation, value: Value) -> Result<Value> {
    match operation {
        Operation::Instance => match value.get("instances") {
            Some(Value::Null) => Ok(json!({"instances":null})),
            Some(row) => project(row, INSTANCE_FIELDS),
            None => Err("Vast single-instance reply lacks instances field".into()),
        },
        Operation::Offers => value
            .get("offers")
            .and_then(Value::as_array)
            .ok_or("Vast offer reply lacks an offer list")?
            .iter()
            .map(|v| project(v, OFFER_FIELDS))
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Operation::Create => {
            let success = value
                .get("success")
                .and_then(Value::as_bool)
                .ok_or("Vast create reply lacks success; reconcile instance label before retry")?;
            if success {
                let contract = value
                    .get("new_contract")
                    .and_then(Value::as_u64)
                    .filter(|n| *n > 0)
                    .ok_or(
                        "Vast create success lacks contract; reconcile instance label before retry",
                    )?;
                Ok(json!({"success":true,"new_contract":contract}))
            } else {
                Ok(
                    json!({"success":false,"msg":"Vast declined create; provider response details suppressed"}),
                )
            }
        }
        Operation::Destroy => {
            if value.get("success") == Some(&json!(true)) {
                Ok(json!({"success":true}))
            } else {
                Err("Vast destroy did not confirm success; verify inventory".into())
            }
        }
        Operation::Instances => Err("instance inventory requires pagination".into()),
    }
}

/// Fetch every page or refuse the entire inventory. Partial inventory must
/// never lower the admission count, including malformed or repeating tokens.
fn inventory<F: FnMut(&Request) -> Result<Value>>(
    mut request: Request,
    mut send: F,
) -> Result<Value> {
    let mut rows = vec![];
    let mut tokens = HashSet::new();
    let mut ids = HashSet::new();
    for _ in 0..1000 {
        let page = send(&request)?;
        let items = page
            .get("instances")
            .and_then(Value::as_array)
            .ok_or("Vast inventory page lacks instances array; partial inventory refused")?;
        for row in items {
            let row = project(row, INSTANCE_FIELDS)?;
            let id = row["id"].as_u64().ok_or("invalid inventory ID")?;
            if !ids.insert(id) {
                return Err(
                    "Vast inventory repeated an instance ID; partial inventory refused".into(),
                );
            }
            rows.push(row);
        }
        match page.get("next_token") {
            None | Some(Value::Null) => return Ok(Value::Array(rows)),
            Some(Value::String(token)) if token.is_empty() => return Ok(Value::Array(rows)),
            Some(Value::String(token)) => {
                if !tokens.insert(token.clone()) {
                    return Err(
                        "Vast inventory pagination repeated a token; partial inventory refused"
                            .into(),
                    );
                }
                request.query.retain(|(k, _)| k != "after_token");
                request.query.push(("after_token".into(), json!(token)));
            }
            _ => {
                return Err(
                    "Vast inventory pagination token is malformed; partial inventory refused"
                        .into(),
                )
            }
        }
    }
    Err("Vast inventory exceeded pagination bound; partial inventory refused".into())
}

pub fn run(args: &[String]) -> Result<Value> {
    let (op, request) = parse(args)?;
    let key = credentials()?;
    if op == Operation::Instances {
        inventory(request, |r| send(r, &key))
    } else {
        normalize(&op, send(&request, &key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).into()).collect()
    }

    #[test]
    fn runner_search_matches_source_aliases_order_storage_and_limits() -> Result<()> {
        let (op,r)=parse(&args(&["search","offers","gpu_name=RTX_5090 num_gpus=1 datacenter=true verified=true rentable=true reliability>0.99 cuda_vers>=12.8 disk_space>=150 inet_down>=100","--storage","150","-o","dph_total","--limit","7","--raw"]))?;
        assert_eq!(op, Operation::Offers);
        assert_eq!(r.method, "POST");
        assert_eq!(r.path, "/api/v0/bundles/");
        let q = r.body.ok_or("body missing")?;
        assert_eq!(q["gpu_name"]["eq"], "RTX 5090");
        assert_eq!(q["cuda_max_good"]["gte"], "12.8");
        assert_eq!(q["reliability"]["gt"], "0.99");
        assert_eq!(q["external"]["eq"], false);
        assert_eq!(q["limit"], 7);
        assert_eq!(q["allocated_storage"], 150.);
        assert_eq!(q["order"], json!([["dph_total", "asc"]]));
        Ok(())
    }
    #[test]
    fn create_keeps_native_container_flags_and_never_discloses_response_keys() -> Result<()> {
        let (_, r) = parse(&args(&[
            "create",
            "instance",
            "7001",
            "--image",
            "cuda-devel",
            "--disk",
            "150",
            "--ssh",
            "--direct",
            "--label",
            "uor-codex-fixture",
            "--env",
            "-e A='x y=z' -p 22:22/tcp",
            "--onstart-cmd",
            "printf native",
            "--raw",
        ]))?;
        let b = r.body.ok_or("body missing")?;
        assert_eq!(b["env"]["A"], "x y=z");
        assert_eq!(b["env"]["-p 22:22/tcp"], "1");
        assert_eq!(b["runtype"], "ssh_direc ssh_proxy");
        assert_eq!(b["onstart"], "printf native");
        let response = normalize(
            &Operation::Create,
            json!({"success":true,"new_contract":8001,"api_key":"SENTINEL_SECRET","msg":"SENTINEL_SECRET"}),
        )?;
        assert_eq!(response, json!({"success":true,"new_contract":8001}));
        assert!(!response.to_string().contains("SENTINEL"));
        Ok(())
    }
    #[test]
    fn pagination_fetches_all_pages_and_refuses_partial_inventory() -> Result<()> {
        let (_, r) = parse(&args(&["show", "instances", "--raw"]))?;
        let mut calls = 0;
        let result = inventory(r.clone(), |request| {
            calls += 1;
            match calls {
                1 => Ok(
                    json!({"instances":[{"id":1,"num_gpus":1,"dph_total":0.536,"label":"a","extra_env":{"KEY":"SECRET"}}],"next_token":"page2"}),
                ),
                2 => {
                    assert_eq!(
                        request.query.last(),
                        Some(&("after_token".into(), json!("page2")))
                    );
                    Ok(
                        json!({"instances":[{"id":2,"num_gpus":2,"dph_total":1.072}],"next_token":null}),
                    )
                }
                _ => Err("unexpected call".into()),
            }
        })?;
        assert_eq!(result.as_array().map(Vec::len), Some(2));
        assert!(!result.to_string().contains("SECRET"));
        assert!(inventory(r.clone(), |_| Ok(
            json!({"instances":[],"next_token":"repeat"})
        ))
        .is_err());
        assert!(inventory(r, |_| Ok(json!({"error":"authentication failed"}))).is_err());
        Ok(())
    }
    #[test]
    fn single_show_and_destroy_match_rest_and_missing_shape_refuses() -> Result<()> {
        let (_, show) = parse(&args(&["show", "instance", "8001", "--raw"]))?;
        assert_eq!(
            url(&show),
            "https://console.vast.ai/api/v0/instances/8001/?owner=me"
        );
        let (_, destroy) = parse(&args(&["destroy", "instance", "8001"]))?;
        assert_eq!(destroy.method, "DELETE");
        assert_eq!(destroy.body, Some(json!({})));
        assert_eq!(
            normalize(&Operation::Instance, json!({"instances":null}))?,
            json!({"instances":null})
        );
        assert!(normalize(
            &Operation::Create,
            json!({"success":true,"api_key":"SECRET"})
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn malformed_cap_fields_refuse_the_entire_instance_inventory() -> Result<()> {
        let (_, request) = parse(&args(&["show", "instances", "--raw"]))?;
        for bad in [
            json!({"id":2,"num_gpus":1}),
            json!({"id":2,"dph_total":0.536}),
            json!({"id":2,"num_gpus":-1,"dph_total":0.536}),
            json!({"id":2,"num_gpus":1.5,"dph_total":0.536}),
            json!({"id":2,"num_gpus":"1","dph_total":0.536}),
            json!({"id":2,"num_gpus":1,"dph_total":-0.536}),
            json!({"id":2,"num_gpus":1,"dph_total":"NaN"}),
            json!({"id":2,"num_gpus":1,"dph_total":null}),
        ] {
            assert!(normalize(&Operation::Instance, json!({"instances":bad.clone()})).is_err());
            let mut page = 0;
            let result = inventory(request.clone(), |_| {
                page += 1;
                Ok(if page == 1 {
                    json!({"instances":[{"id":1,"num_gpus":1,"dph_total":0.536}],"next_token":"second"})
                } else {
                    json!({"instances":[bad.clone()],"next_token":null})
                })
            });
            assert!(result.is_err());
            assert_eq!(page, 2);
        }
        assert!(normalize(
            &Operation::Instance,
            json!({"instances":{"id":3,"num_gpus":0,"dph_total":0.}})
        )
        .is_ok());
        Ok(())
    }
    #[test]
    fn credentials_and_payload_stay_in_stdin_config_with_no_retry_options() -> Result<()> {
        let (_, r) = parse(&args(&[
            "create",
            "instance",
            "1",
            "--image",
            "native",
            "--label",
            "fixture",
            "--onstart-cmd",
            "echo \"a\"\nnext",
        ]))?;
        let config = curl_config(&r, "FIXTURE_KEY");
        assert!(config.contains("Authorization: Bearer FIXTURE_KEY"));
        assert!(config.contains("data = "));
        assert!(!config.contains("retry ="));
        assert!(parse(&args(&["create", "instance", "1", "--image", "native"])).is_err());
        assert!(parse(&args(&[
            "search",
            "offers",
            "gpu_name=RTX_5090",
            "--unknown",
            "value"
        ]))
        .is_err());
        assert!(parse(&args(&["destroy", "instance", "1;echo SECRET"])).is_err());
        Ok(())
    }
}

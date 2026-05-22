pub mod socket_protocol;
pub mod client;
pub mod server_pool;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use anyhow::Result;
pub use socket_protocol::{ProxyRequest, ProxyResponse};
use server_pool::ServerPool;

pub fn default_socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/livefolders/mcp.sock")
}

pub fn default_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/livefolders/mcp-servers.yaml")
}

pub fn run_proxy(socket_path: PathBuf, config_path: PathBuf) -> Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pool = Arc::new(ServerPool::new(config_path));
    let stop = Arc::new(AtomicBool::new(false));
    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("mcp-proxy listening at {}", socket_path.display());

    for stream in listener.incoming() {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(s) => {
                let pool = Arc::clone(&pool);
                let stop = Arc::clone(&stop);
                std::thread::spawn(move || handle_connection(s, pool, stop));
            }
            Err(e) => tracing::warn!("mcp-proxy: accept error: {}", e),
        }
    }
    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

fn handle_connection(stream: UnixStream, pool: Arc<ServerPool>, stop: Arc<AtomicBool>) {
    let writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut writer = std::io::BufWriter::new(writer);
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };
        let req: ProxyRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = ProxyResponse::failure(format!("parse error: {}", e));
                let _ = writeln!(writer, "{}", serde_json::to_string(&resp).unwrap());
                let _ = writer.flush();
                continue;
            }
        };

        let resp = match req {
            ProxyRequest::Call { server, tool, args } => {
                match pool.call(&server, &tool, args) {
                    Ok(result) => ProxyResponse::success(result),
                    Err(e) => ProxyResponse::failure(e.to_string()),
                }
            }
            ProxyRequest::Status => {
                let servers = pool.running_servers().join("\n");
                ProxyResponse::success(servers)
            }
            ProxyRequest::Stop => {
                stop.store(true, Ordering::Relaxed);
                let resp = ProxyResponse::success("stopping".to_string());
                let _ = writeln!(writer, "{}", serde_json::to_string(&resp).unwrap());
                let _ = writer.flush();
                break;
            }
        };

        let _ = writeln!(writer, "{}", serde_json::to_string(&resp).unwrap());
        let _ = writer.flush();
    }
}

/// Send a single request to a running proxy and return the response.
pub fn proxy_call(socket_path: &PathBuf, req: &ProxyRequest) -> Result<ProxyResponse> {
    let stream = UnixStream::connect(socket_path)
        .map_err(|_| anyhow::anyhow!("mcp-proxy is not running — start it with: livefolders mcp proxy"))?;
    let mut writer = std::io::BufWriter::new(stream.try_clone()?);
    writeln!(writer, "{}", serde_json::to_string(req)?)?;
    writer.flush()?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    Ok(serde_json::from_str(line.trim())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::os::unix::fs::PermissionsExt;

    fn fake_mcp_script_path(tmp: &tempfile::TempDir) -> std::path::PathBuf {
        let script = tmp.path().join("fake.py");
        std::fs::write(&script, r#"#!/usr/bin/env python3
import sys,json
def s(o): sys.stdout.write(json.dumps(o)+"\n"); sys.stdout.flush()
for line in sys.stdin:
    line=line.strip()
    if not line: continue
    req=json.loads(line); method=req.get("method",""); id_=req.get("id")
    if method=="initialize": s({"jsonrpc":"2.0","id":id_,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"f","version":"0"}}})
    elif method=="notifications/initialized": pass
    elif method=="tools/call": s({"jsonrpc":"2.0","id":id_,"result":{"content":[{"type":"text","text":"pong"}],"isError":False}})
"#).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        script
    }

    #[test]
    fn proxy_call_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let script = fake_mcp_script_path(&tmp);
        let cfg_path = tmp.path().join("mcp-servers.yaml");
        std::fs::write(&cfg_path, format!(
            "servers:\n  fake:\n    command: python3\n    args: [\"{}\"]\n",
            script.display()
        )).unwrap();
        let socket_path = tmp.path().join("proxy.sock");

        let sp = socket_path.clone();
        let cp = cfg_path.clone();
        std::thread::spawn(move || { let _ = run_proxy(sp, cp); });
        std::thread::sleep(std::time::Duration::from_millis(150));

        let mut stream = UnixStream::connect(&socket_path).unwrap();
        let req = serde_json::json!({"op":"call","server":"fake","tool":"ping","args":{}});
        writeln!(stream, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        let mut reader = BufReader::new(stream);
        let mut resp_line = String::new();
        reader.read_line(&mut resp_line).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["ok"], true, "response: {:?}", resp);
        assert_eq!(resp["result"], "pong");
    }

    #[test]
    fn proxy_status_lists_running_servers() {
        let tmp = tempfile::tempdir().unwrap();
        let script = fake_mcp_script_path(&tmp);
        let cfg_path = tmp.path().join("mcp-servers.yaml");
        std::fs::write(&cfg_path, format!(
            "servers:\n  fake:\n    command: python3\n    args: [\"{}\"]\n",
            script.display()
        )).unwrap();
        let socket_path = tmp.path().join("proxy2.sock");

        let sp = socket_path.clone();
        let cp = cfg_path.clone();
        std::thread::spawn(move || { let _ = run_proxy(sp, cp); });
        std::thread::sleep(std::time::Duration::from_millis(150));

        // First do a call to start the server
        {
            let mut stream = UnixStream::connect(&socket_path).unwrap();
            let req = serde_json::json!({"op":"call","server":"fake","tool":"ping","args":{}});
            writeln!(stream, "{}", serde_json::to_string(&req).unwrap()).unwrap();
            let mut buf = String::new();
            BufReader::new(stream).read_line(&mut buf).unwrap();
        }

        // Then check status
        let mut stream = UnixStream::connect(&socket_path).unwrap();
        let req = serde_json::json!({"op":"status"});
        writeln!(stream, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        let mut resp_line = String::new();
        BufReader::new(stream).read_line(&mut resp_line).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["ok"], true);
        assert!(resp["result"].as_str().unwrap_or("").contains("fake"));
    }
}

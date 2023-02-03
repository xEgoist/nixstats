use axum::{
    extract::ws::{self, CloseFrame, Message, WebSocket, WebSocketUpgrade},
    http::Request,
    response::Response,
    routing::get,
    Router,
};
use hyper::body::Buf;
use hyper::Body;
use serde::Deserialize;

#[derive(Debug)]
enum Error {
    ParseError(std::net::AddrParseError),
    ServeError(hyper::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/ws", get(handler));
    axum::Server::bind(&"0.0.0.0:3000".parse().map_err(Error::ParseError)?)
        .serve(app.into_make_service())
        .await
        .map_err(Error::ServeError)?;
    Ok(())
}
async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };
        match msg {
            axum::extract::ws::Message::Text(t) => {
                println!("RECEIVED {}", &t);
                let pr_number: u32 = t.parse().unwrap();
                let sha = request_pr_sha(pr_number).await.unwrap();
                let mut set = tokio::task::JoinSet::new();
                for branch in [
                    Branch::NixosUnstable,
                    Branch::NixosUnstableSmall,
                    Branch::NixpkgsUnstable,
                    Branch::Master,
                ] {
                    set.spawn(get_status(branch, sha.clone()));
                }
                let mut ret_vec = vec![];
                while let Some(res) = set.join_next().await {
                    let stat = res.unwrap();
                    let status = stat.unwrap();
                    if status == BranchStatus::Diverged || status == BranchStatus::Ahead {
                        ret_vec.push(0);
                    } else {
                        ret_vec.push(1);
                    }
                }
                let _ = socket.send(Message::Binary(ret_vec)).await;
                let exit = CloseFrame {
                    code: axum::extract::ws::close_code::NORMAL,
                    reason: "".into(),
                };
                let _ = socket.send(Message::Close(Some(exit))).await;
            }
            axum::extract::ws::Message::Close(c) => {
                if let Some(closed) = c {
                    if closed.code == ws::close_code::NORMAL {
                        println!(
                            "Connection Closed Normally {}\n Reason:{}",
                            closed.code, closed.reason
                        );
                    }
                }
            }
            _ => {
                println!("Unsupported Message Type");
                let exit = CloseFrame {
                    code: axum::extract::ws::close_code::INVALID,
                    reason: "UnImplemented Format You Silly Goose".into(),
                };
                let _ = socket.send(Message::Close(Some(exit))).await;
            }
        }
    }
}

#[derive(Debug)]
pub enum Branch {
    NixosUnstable,
    NixosUnstableSmall,
    NixpkgsUnstable,
    Master,
}
impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NixosUnstable => write!(f, "nixos-unstable"),
            Self::NixpkgsUnstable => write!(f, "nixpkgs-unstable"),
            Self::NixosUnstableSmall => write!(f, "nixos-unstable-small"),
            Branch::Master => write!(f, "master"),
        }
    }
}
#[derive(PartialEq, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum BranchStatus {
    Diverged,
    Ahead,
    Behind,
    Identical,
}

#[derive(Deserialize)]
struct BranchResponse {
    status: BranchStatus,
}

type HyperResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Deserialize, Debug)]
pub struct PRResponse {
    merge_commit_sha: String,
}
async fn request_pr_sha(pr: u32) -> HyperResult<String> {
    let url =
        format!("https://api.github.com/repos/nixos/nixpkgs/pulls/{pr}").parse::<hyper::Uri>()?;
    let mut res = handle_request(&url).await?;
    let body = hyper::body::aggregate(res.body_mut()).await?;
    let res: PRResponse = serde_json::from_reader(body.reader())?;
    Ok(res.merge_commit_sha)
}

async fn get_status(branch: Branch, merge_commit_sha: String) -> HyperResult<BranchStatus> {
    let url = format!(
        "https://api.github.com/repos/nixos/nixpkgs/compare/{}...{}",
        branch, merge_commit_sha
    )
    .parse::<hyper::Uri>()?;
    let mut res = handle_request(&url).await?;
    // let body = hyper::body::aggregate(res.body_mut()).await?;
    let body = hyper::body::to_bytes(res.body_mut()).await?;
    // let body_str = std::str::from_utf8(&body)?;
    // debug
    // println!("@@@@@@@@ {} @@@@@@@@", body_str);
    let branch: BranchResponse = serde_json::from_slice(&body)?;
    // try to parse as json with serde_json
    // let branch: BranchResponse = serde_json::from_reader(body.reader())?;
    println!("Response status: {}", res.status());
    Ok(branch.status)
}
// TODO: Pass the client here so we don't keep making new ones for each request
async fn handle_request(url: &hyper::Uri) -> HyperResult<Response<Body>> {
    let ssl = hyper_openssl::HttpsConnector::new().unwrap();
    let client = hyper::Client::builder().build::<_, Body>(ssl);

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::USER_AGENT, "nixstats")
        .header(hyper::header::ACCEPT, "application/vnd.github+json")
        .header(
            hyper::header::AUTHORIZATION,
            "Bearer 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",
        )
        .body(Body::empty())?;
    let res: Response<Body> = client.request(req).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_pr() {
        assert_eq!(
            get_status(
                Branch::NixosUnstableSmall,
                "14c5d226877aa90f656895ab7ab2badc4af03268".to_owned(),
            )
            .await
            .unwrap(),
            BranchStatus::Behind
        )
    }
    #[tokio::test]
    async fn pr_test() {
        assert_eq!(
            request_pr_sha("193766".parse().unwrap()).await.unwrap(),
            "14c5d226877aa90f656895ab7ab2badc4af03268".to_owned()
        );
    }
}

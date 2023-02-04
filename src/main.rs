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
    ParseIntError(std::num::ParseIntError),
    ServeError(hyper::Error),
}

type Result<T> = std::result::Result<T, Error>;
static API_KEY: &'static str = env!("API_KEY");

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
                let pr_number: Result<u32> = t.parse().map_err(Error::ParseIntError);
                if let Err(_) = pr_number {
                    let exit = CloseFrame {
                        code: axum::extract::ws::close_code::INVALID,
                        reason: "PR number should only consist of numbers".into(),
                    };
                    let _ = socket.send(Message::Close(Some(exit))).await;
                    continue;
                }
                let sha = request_pr_sha(pr_number.unwrap()).await;
                if let Ok(sha_val) = sha {
                    let mut set = tokio::task::JoinSet::new();
                    for branch in [
                        Branch::NixosUnstable,
                        Branch::NixosUnstableSmall,
                        Branch::NixpkgsUnstable,
                        Branch::Master,
                        Branch::Nixos2211,
                    ] {
                        set.spawn(get_status(branch, sha_val.clone()));
                    }
                    let mut ret = [0; 5];

                    while let Some(res) = set.join_next().await {
                        let stat = res.unwrap();
                        let status = stat.unwrap();
                        if status.1 != BranchStatus::Diverged && status.1 != BranchStatus::Ahead {
                            ret[status.0 as usize] = 1;
                        }
                    }
                    let _ = socket.send(Message::Binary(ret.to_vec())).await;
                } else {
                    let exit = CloseFrame {
                        code: axum::extract::ws::close_code::INVALID,
                        reason: "Error: Unable to find the PR provided".into(),
                    };
                    let _ = socket.send(Message::Close(Some(exit))).await;
                    continue;
                }
                // Close normally when done
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

#[derive(Debug, PartialEq)]
pub enum Branch {
    NixosUnstable = 0,
    NixosUnstableSmall = 2,
    NixpkgsUnstable = 4,
    Nixos2211 = 3,
    Master = 1,
}
impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NixosUnstable => write!(f, "nixos-unstable"),
            Self::NixpkgsUnstable => write!(f, "nixpkgs-unstable"),
            Self::NixosUnstableSmall => write!(f, "nixos-unstable-small"),
            Self::Master => write!(f, "master"),
            Self::Nixos2211 => write!(f, "nixos-22.11"),
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

async fn get_status(
    branch: Branch,
    merge_commit_sha: String,
) -> HyperResult<(Branch, BranchStatus)> {
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
    let ret: BranchResponse = serde_json::from_slice(&body)?;
    // try to parse as json with serde_json
    // let branch: BranchResponse = serde_json::from_reader(body.reader())?;
    println!("Response status: {}", res.status());
    Ok((branch, ret.status))
}
// TODO: Pass the client here so we don't keep making new ones for each request
async fn handle_request(url: &hyper::Uri) -> HyperResult<Response<Body>> {
    let ssl = hyper_openssl::HttpsConnector::new()?;
    let client = hyper::Client::builder().build::<_, Body>(ssl);

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::USER_AGENT, "nixstats")
        .header(hyper::header::ACCEPT, "application/vnd.github+json")
        .header(hyper::header::AUTHORIZATION, format!("Bearer {API_KEY}"))
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
            (Branch::NixosUnstableSmall, BranchStatus::Behind)
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

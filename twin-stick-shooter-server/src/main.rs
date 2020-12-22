use bytes::BytesMut;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use hyper::body::Sender;
use hyper::header::{
    HeaderValue, CONNECTION, CONTENT_TYPE, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE,
};
use hyper::server::conn::Http;
use hyper::upgrade::Upgraded;
use hyper::{Body, Request, Response, StatusCode};
use sha1::{Digest, Sha1};
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_compat_02::{FutureExt, IoCompat};
use tokio_tungstenite::tungstenite::protocol::Role;
use webrtc_unreliable::{MessageResult, MessageType, SessionEndpoint};

#[derive(Debug, StructOpt)]
#[structopt(name = "twin-stick-shooter-server")]
struct Opt {
    /// Listen address for HTTP connections over TCP.
    #[structopt(long)]
    http_listen_addr: String,

    #[structopt(long)]
    webrtc_listen_addr: SocketAddr,

    #[structopt(long)]
    webrtc_public_addr: SocketAddr,

    /// Path to static content to serve on otherwise unmapped URLs.
    #[structopt(long)]
    static_content_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opt = Opt::from_args();

    let webrtc_server =
        webrtc_unreliable::Server::new(opt.webrtc_listen_addr, opt.webrtc_public_addr).await?;
    let session_endpoint = webrtc_server.session_endpoint();
    tokio::spawn(handle_webrtc_sessions(webrtc_server));

    let listener = TcpListener::bind(&opt.http_listen_addr).await?;
    let shared_state = Arc::new(SharedState {
        opt,
        webrtc_session_endpoint: Mutex::new(session_endpoint),
    });
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_http_connection(Arc::clone(&shared_state), stream));
    }
}

async fn handle_webrtc_sessions(mut webrtc_server: webrtc_unreliable::Server) {
    loop {
        let MessageResult {
            message,
            message_type,
            remote_addr,
        } = webrtc_server
            .recv()
            .await
            .expect("error in webrtc_server.recv()");
        println!(
            "received a message: {{ message: {:?}, message_type: {:?}, remote_addr: {:?} }}",
            message.as_slice(),
            message_type,
            remote_addr,
        );
        drop(message);

        webrtc_server
            .send(
                b"the server deigns to reply",
                MessageType::Text,
                &remote_addr,
            )
            .await
            .expect("error in webrtc_server.send()");
    }
}

struct SharedState {
    opt: Opt,
    webrtc_session_endpoint: Mutex<SessionEndpoint>,
}

async fn handle_http_connection(shared_state: Arc<SharedState>, stream: TcpStream) {
    Http::new()
        .serve_connection(
            IoCompat::new(stream),
            hyper::service::service_fn(|req: Request<Body>| {
                let shared_state = Arc::clone(&shared_state);
                async move { handle_http_request(shared_state, req).await }
            }),
        )
        .with_upgrades()
        .compat()
        .await
        .expect("error serving HTTP connection");
}

async fn handle_http_request(
    shared_state: Arc<SharedState>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::http::Error> {
    if req.uri().path() == "/special" {
        Ok(Response::new("you have reached the special URL".into()))
    } else if req.uri().path() == "/websocket" {
        upgrade_http_request_to_websocket(req).await
    } else if req.uri().path() == "/webrtc-offer" {
        handle_webrtc_offer(&shared_state, req).await
    } else {
        serve_static_content(&shared_state.opt.static_content_path, req).await
    }
}

async fn handle_webrtc_offer(
    shared_state: &Arc<SharedState>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::http::Error> {
    let reply = shared_state
        .webrtc_session_endpoint
        .lock()
        .await
        .session_request(req.into_body())
        .await
        .map_err(|e| -> hyper::http::Error {
            panic!("webrtc session error: {}", e);
        })?;
    Ok(Response::new(reply.into()))
}

async fn upgrade_http_request_to_websocket(
    req: Request<Body>,
) -> Result<Response<Body>, hyper::http::Error> {
    const WEBSOCKET_HANDSHAKE_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut resp = Response::new(Body::empty());
    if !req.headers().contains_key(UPGRADE) {
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(resp);
    }
    let hash_bytes = if let Some(sec_websocket_key) = req.headers().get(SEC_WEBSOCKET_KEY) {
        let mut hasher = Sha1::new();
        hasher.update(sec_websocket_key);
        hasher.update(WEBSOCKET_HANDSHAKE_GUID);
        hasher.finalize()
    } else {
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(resp);
    };
    let hash_base64 = base64::encode(&hash_bytes[..]);

    tokio::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                if let Err(e) = handle_websocket(upgraded).await {
                    eprintln!("websocket handling error: {:?}", e);
                }
            }
            Err(e) => eprintln!("upgrade error: {}", e),
        }
    });

    *resp.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    resp.headers_mut()
        .insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    resp.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("websocket"));
    resp.headers_mut().insert(
        SEC_WEBSOCKET_ACCEPT,
        HeaderValue::from_str(&hash_base64).unwrap(),
    );
    Ok(resp)
}

async fn handle_websocket(upgraded: Upgraded) -> anyhow::Result<()> {
    let ws = tokio_tungstenite::WebSocketStream::from_raw_socket(
        IoCompat::new(upgraded),
        Role::Server,
        None,
    )
    .await;
    let (mut sink, mut stream) = ws.split();
    while let Some(msg) = stream.next().await {
        if let Err(tungstenite::error::Error::ConnectionClosed) = msg {
            break;
        }
        match sink.send(msg?).await {
            Ok(()) => (),
            Err(tungstenite::error::Error::ConnectionClosed) => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

async fn serve_static_content(
    static_content_path: &Path,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::http::Error> {
    let mut local_path = static_content_path.to_path_buf();
    local_path.extend(req.uri().path().split('/'));
    if req.uri().path().ends_with("/") {
        local_path.push("index.html");
    }
    match File::open(&local_path).await {
        Ok(file) => {
            let (sender, body) = Body::channel();
            tokio::spawn(stream_file_to_sender(file, sender));
            let mut resp = Response::builder().status(StatusCode::OK);
            set_content_type_header_for_extension(&mut resp, local_path.extension());
            resp.body(body)
        }
        Err(_) => {
            // Treat all errors as absence.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap())
        }
    }
}

async fn stream_file_to_sender(mut file: File, mut sender: Sender) {
    loop {
        let mut buf = BytesMut::with_capacity(4096); // TODO: tune buffer size
        buf.resize(4096, 0);
        let n = file
            .read(buf.as_mut())
            .await
            .expect("error reading from static content");
        if n == 0 {
            break;
        }
        buf.resize(n, 0);
        sender
            .send_data(buf.freeze())
            .await
            .expect("error writing static content to response body");
    }
}

fn set_content_type_header_for_extension(
    resp: &mut hyper::http::response::Builder,
    extension: Option<&OsStr>,
) {
    if let Some(content_type) = extension
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext {
            "css" => Some("text/css; charset=utf-8"),
            "html" => Some("text/html; charset=utf-8"),
            "js" => Some("text/javascript; charset=utf-8"),
            "json" => Some("application/json"),
            "wasm" => Some("application/wasm"),
            _ => None,
        })
    {
        resp.headers_mut()
            .unwrap()
            .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    }
}

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use mime_guess::from_path;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::runtime::Runtime;

const DEFAULT_DIR: &str = "web-export";
const DEFAULT_ADDR: &str = "127.0.0.1";

#[derive(Parser)]
pub struct Args {
    /// Directory to serve (defaults to web-export)
    #[arg(long, default_value = "web-export")]
    dir: String,
    /// Port to listen on (defaults to 8080)
    #[arg(long, default_value = "8080")]
    port: u16,
}
impl Args {
    pub fn new(dir: Option<String>, port: Option<u16>) -> Self {
        Self {
            dir: dir.unwrap_or_else(|| DEFAULT_DIR.to_string()),
            port: port.unwrap_or(8080),
        }
    }
}

/// web_serverコマンド本体
pub fn run(args: &Args) {
    let dir = &args.dir;
    let root = PathBuf::from(dir);

    if !root.exists() {
        if dir == DEFAULT_DIR {
            eprintln!(
                "❌ '{}' directory not found. Please run 'sgdktool web-export' first.",
                DEFAULT_DIR
            );
        } else {
            eprintln!("❌ Directory '{}' does not exist.", dir);
        }
        std::process::exit(1);
    }

    let port = args.port;

    println!("Starting web server for directory: {}", root.display());
    println!("Listening on http://{}:{}/", DEFAULT_ADDR, port);

    // tokio runtimeを生成してサーバ起動
    Runtime::new().unwrap().block_on(async {
        let addr: SocketAddr = format!("{}:{}", DEFAULT_ADDR, port).parse().unwrap();
        let root = root.clone();

        let make_svc = make_service_fn(move |_| {
            let root = root.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    serve_static_with_headers(req, root.clone())
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}

async fn serve_static_with_headers(
    req: Request<Body>,
    root: PathBuf,
) -> Result<Response<Body>, hyper::Error> {
    let mut path = req.uri().path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }
    let file_path = root.join(&path);

    let mut response = if file_path.exists() && file_path.is_file() {
        match fs::read(&file_path) {
            Ok(contents) => {
                let mime = from_path(&file_path).first_or_octet_stream();
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime.as_ref())
                    .body(Body::from(contents))
                    .unwrap()
            }
            Err(_) => response_404(),
        }
    } else {
        response_404()
    };

    // COOP/COEPヘッダを付与
    let headers = response.headers_mut();
    headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        "require-corp".parse().unwrap(),
    );

    Ok(response)
}

fn response_404() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(Body::from("404 Not Found"))
        .unwrap()
}

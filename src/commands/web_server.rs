use clap::Parser;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use mime_guess::from_path;
use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
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
        let listener = TcpListener::bind(addr).await.unwrap();
        let shared_root = Arc::new(root.clone());

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let service = StaticFileService {
                root: Arc::clone(&shared_root),
            };

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Error serving connection: {}", err);
                }
            });
        }
    });
}

struct StaticFileService {
    root: Arc<PathBuf>,
}

impl Service<Request<hyper::body::Incoming>> for StaticFileService {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<hyper::body::Incoming>) -> Self::Future {
        let root = self.root.clone();

        Box::pin(async move {
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
                            .body(Full::new(Bytes::from(contents)))
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
        })
    }
}

fn response_404() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from("404 Not Found")))
        .unwrap()
}

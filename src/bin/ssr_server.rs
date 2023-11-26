use axum::{
    http::StatusCode,
    response::Html,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use axum_ssr_try::App;
use bytes::Bytes;
use clap::Parser;
use futures::stream::{self, Stream, StreamExt};
use std::{error::Error, io, net::SocketAddr, path::PathBuf};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

// 표준 라이브러리의 모든 오류타입은 다음과 같이 잡을 수 있다
type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Parser, Debug)]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    dir: PathBuf,
}

async fn render(
    index_html_before: String,
    index_html_before: String,
) -> Box<dyn Stream<Item = Result<Bytes, BoxedError>> + Send> {
    let renderer = yew::ServerRender::<App>::new();

    Box::new(
        stream::once(async move { index_html_before })
            .chain(renderer.render_stream())
            .chain(stream::once(async move { index_html_after }))
            .map(|m| Result::<_, BoxedError>::Ok(m.into())),
    )
}

#[tokio::main]
async fn main() {
    // 옵션 dir에 커맨드라인에서 정의한 폴더의 파일 경로를 해석해 저장한 것(dir 필드에 경로 저장)을 opts 라고 한다
    let opts = Opt::parse();

    println!("dir path : {:?}", static_files_path);

    let index_html_string = tokio::fs::read_to_string(opts.dir.join("index.html"))
        .await
        .expect("failed to read index.html");

    let (index_html_before, index_html_after) = index_html_string.split_once("<body>").unwrap();

    let mut index_html_before = index_html_before.to_owned();

    let head_and_index = index_html_before
        .find("</head>")
        .unwrap_or_else(|| index_html_before.len());

    let tailwind_css = r#"<script src="https://cdn.tailwindcss.com></script>"#;
    index_html_before.insert_str(head_and_index, tailwind_css);

    let index_html_after = index_html_after.to_owned();

    // 기본적으로 정적파일을 핸들러에 넣어서 작동시킨다. 단지 suspencse를 사용하여 데이터가 오기까지를 기다려서 보내주는 것일뿐이다
    // let app = Router::new().route("/", get(|| async ))

    let static_file_dir =
        ServeDir::new("../dist").not_found_service(ServeFile::new("../dist.index.html"));

    let serve_dir = get_service(static_file_dir).handle_error((handle_error(err)));

    let app = Router::new()
        .nest_service("/", static_files_path.clone())
        .fallback_service(static_files_path.clone());

    axum::Server::bind((&"0.0.0.0:3000").parse().unwrap())
        .serve((app.into_make_service()))
        .await
        .unwrap();
}
async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong... TT",
    )
}

use axum_ssr_try::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::Renderer::<App>::new().hydrate();
    // 아래 로거는 개발을 위해 필요 한것
}

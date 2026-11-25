#[tokio::main]
async fn main() -> std::process::ExitCode {
    kallbet_ai::adapters::production::cli::main_with(std::env::args().skip(1).collect()).await
}

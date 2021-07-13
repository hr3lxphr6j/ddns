use clap::{App, Arg};
use ddns::Application;
use tokio;

#[tokio::main]
async fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
    let matches = App::new("ddns")
        .version("0.0.1")
        .author("Chigusa. chigusa@chigusa.moe")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();
    let config = matches.value_of("config").unwrap_or("ddns.toml");
    let app = Application::new(config).unwrap();
    match app.do_update().await {
        Ok(_) => log::info!("Done."),
        Err(_) => {
            log::error!("Failed.");
            std::process::exit(1)
        }
    }
}

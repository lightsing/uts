use reqwest::Client;
use std::sync::LazyLock;

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(concat!("uts/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build HTTP client")
});

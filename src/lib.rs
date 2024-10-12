use std::path::PathBuf;

#[derive(Clone)]
pub struct Kaffe {
    pub client_build_dir: PathBuf,
    pub client_bundle_path: String,
    pub server_bundle_path: String,
    pub server_port: u16,
}

impl Kaffe {
    pub fn new(
        client_build_dir: impl Into<PathBuf>,
        client_bundle_path: impl Into<String>,
        server_bundle_path: impl Into<String>,
        server_port: u16,
    ) -> Self {
        Self {
            client_build_dir: client_build_dir.into(),
            client_bundle_path: client_bundle_path.into(),
            server_bundle_path: server_bundle_path.into(),
            server_port,
        }
    }
}

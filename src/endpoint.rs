pub mod endpoint {
    use crate::types::types::HttpMethod;
    use std::fs;
    use std::ops::Add;

    pub struct EndpointHandler {
        endpoints: Vec<Endpoint>,
    }

    impl EndpointHandler {
        pub fn create() -> EndpointHandler {
            return EndpointHandler { endpoints: vec![] };
        }

        pub fn to_provider(&self) -> EndpointProvider {
            return EndpointProvider {
                endpoints: self.endpoints.to_vec(), // performance for many threads?
            };
        }

        pub fn register_assets(&mut self, path: String, mapping: String) {
            let path_str = path.as_str();
            for asset_path_res in fs::read_dir(path_str).unwrap() {
                match asset_path_res {
                    Ok(asset_path) => {
                        if let Some(existing) = self.endpoints.iter().find(|e| {
                            e.path == asset_path.path().into_os_string().into_string().unwrap()
                        }) {
                            println!(
                                "Path {} already registered: {:?}. Skip.",
                                path_str, existing
                            );
                            continue;
                        }
                        if asset_path.file_name().into_string().unwrap() == "index.html" {
                            let full_asset_path =
                                asset_path.path().into_os_string().into_string().unwrap();
                            let mut directory = asset_path
                                .path()
                                .parent()
                                .unwrap()
                                .as_os_str()
                                .to_os_string()
                                .into_string()
                                .unwrap();
                            directory.push_str("/");
                            let endpoint =
                                Endpoint::assets(String::from(full_asset_path), vec![directory]);
                            println!("Registered endpoint: {:?}", endpoint);
                            self.endpoints.push(endpoint);
                        } else {
                            let full_asset_path =
                                asset_path.path().into_os_string().into_string().unwrap();
                            let endpoint = Endpoint::assets(String::from(full_asset_path), vec![]);
                            println!("Registered endpoint: {:?}", endpoint);
                            self.endpoints.push(endpoint);
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    pub struct EndpointProvider {
        endpoints: Vec<Endpoint>,
    }

    impl EndpointProvider {
        pub fn match_endpoint(&self, path: String, method: HttpMethod) -> Option<&Endpoint> {
            println!(
                "Called to resolve endpoint for path {} with method {:?}",
                path, method
            );
            return self.endpoints.iter().find(|e| {
                (e.path == path || e.aliases.contains(&path)) && e.methods.contains(&method)
            });
        }
    }

    #[derive(Debug, Clone)]
    pub struct Endpoint {
        pub path: String,
        pub endpoint_type: EndpointType,
        aliases: Vec<String>,
        methods: Vec<HttpMethod>,
    }

    impl Endpoint {
        pub fn assets(path: String, aliases: Vec<String>) -> Endpoint {
            return Endpoint {
                endpoint_type: EndpointType::Assets,
                path,
                aliases,
                methods: vec![HttpMethod::Get],
            };
        }
    }
    #[derive(Debug, Clone)]
    pub enum EndpointType {
        Assets,
        Resource,
    }
}

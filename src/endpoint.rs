pub mod endpoint {
    use crate::path::path::remap;
    use crate::types::types::HttpMethod;
    use std::fs;
    use std::ops::Add;
    use std::path::Path;

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

        pub fn register_assets(&mut self, location: String, mapping: String) {
            let path = Path::new(location.as_str());
            let absolute_path = match path.is_absolute() {
                true => path.to_path_buf(),
                false => {
                    let cleaned_location = match location.starts_with("./") {
                        true => &location[2..],
                        false => &location,
                    };
                    let current_dir = std::env::current_dir().unwrap();
                    Path::new(&current_dir).join(cleaned_location)
                }
            };
            for local_asset_path_res in fs::read_dir(absolute_path).unwrap() {
                match local_asset_path_res {
                    Ok(local_asset_path) => {
                        let asset_path_str = local_asset_path
                            .path()
                            .into_os_string()
                            .into_string()
                            .unwrap();
                        if let Some(existing) =
                            self.endpoints.iter().find(|e| e.path == asset_path_str)
                        {
                            println!(
                                "Path {} already registered: {:?}. Skip.",
                                asset_path_str, existing
                            );
                            continue;
                        }
                        let full_asset_path = local_asset_path.path();
                        let directory = full_asset_path.parent().unwrap();
                        let remapped_path = remap(
                            &full_asset_path,
                            &directory,
                            Path::new(&mapping),
                        )
                        .as_os_str()
                        .to_os_string()
                        .into_string()
                        .unwrap();

                        if local_asset_path.file_name().into_string().unwrap() == "index.html" {
                            let alias_path = directory.join(Path::new("/")).into_os_string().to_os_string().into_string().unwrap();
                            let endpoint =
                                Endpoint::assets(String::from(remapped_path), vec![alias_path]);
                            println!("Registered endpoint: {:?}", endpoint);
                            self.endpoints.push(endpoint);
                        } else {
                            let endpoint = Endpoint::assets(String::from(remapped_path), vec![]);
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

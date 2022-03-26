pub mod endpoint {
    use crate::path::path::remap;
    use crate::resource::resource::ResourceHandler;
    use crate::types::types::{HttpMethod, HttpRequest};
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    pub struct EndpointHandler {
        endpoints: Vec<Endpoint>,
        resource_handler: HashMap<String, Arc<ResourceHandler>>,
    }

    impl EndpointHandler {
        pub fn create() -> EndpointHandler {
            return EndpointHandler {
                endpoints: vec![],
                resource_handler: HashMap::new(),
            };
        }

        pub fn to_provider(&self) -> EndpointProvider {
            let mut resource_handler_copy: HashMap<String, Arc<ResourceHandler>> = self
                .resource_handler
                .iter()
                .map(|(key, val)| (key.clone(), Arc::clone(val)))
                .collect();
            return EndpointProvider {
                endpoints: self.endpoints.to_vec(), // performance for many threads?
                resource_handler: resource_handler_copy,
            };
        }

        pub fn register_assets(&mut self, location: String, mapping: String) {
            let absolute_path = self
                .map_to_absolute(&location)
                .into_os_string()
                .into_string()
                .unwrap();
            let mapping_corrected = match mapping.starts_with("/") {
                true => mapping,
                false => ["/", &mapping].join(""),
            };
            let endpoint = Endpoint {
                path: mapping_corrected,
                aliases: vec![],
                methods: vec![HttpMethod::Get],
                endpoint_type: EndpointType::Assets(AssetEndpoint {
                    asset_base: absolute_path,
                }),
            };
            self.register_endpoint(endpoint);
        }

        pub fn register_static(&mut self, location: String, mapping: String) {
            let absolute_path = self.map_to_absolute(&location);
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
                        let remapped_path =
                            remap(&full_asset_path, &directory, Path::new(&mapping));
                        let remapped_path_str = remapped_path
                            .as_os_str()
                            .to_os_string()
                            .into_string()
                            .unwrap();

                        if local_asset_path.file_name().into_string().unwrap() == "index.html" {
                            let alias_path = remapped_path
                                .parent()
                                .unwrap()
                                .as_os_str()
                                .to_os_string()
                                .into_string()
                                .unwrap();
                            let endpoint = Endpoint::asset(
                                remapped_path_str,
                                full_asset_path.into_os_string().into_string().unwrap(),
                                vec![alias_path],
                            );
                            self.register_endpoint(endpoint);
                        } else {
                            let endpoint = Endpoint::asset(
                                String::from(remapped_path_str),
                                full_asset_path.into_os_string().into_string().unwrap(),
                                vec![],
                            );
                            self.register_endpoint(endpoint);
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        pub fn register_resource(
            &mut self,
            mapping: String,
            handler_id: String,
            handler: Box<ResourceHandler>,
        ) {
            let mapping_corrected = match mapping.starts_with("/") {
                true => mapping,
                false => ["/", &mapping].join(""),
            };
            let endpoint = Endpoint {
                endpoint_type: EndpointType::Resource(ResourceEndpoint {
                    resource_handler_id: handler_id.clone(),
                }),
                path: mapping_corrected,
                aliases: vec![],
                methods: vec![HttpMethod::Get],
            };
            self.resource_handler
                .insert(handler_id.clone(), Arc::from(handler));
            self.register_endpoint(endpoint);
        }

        fn register_endpoint(&mut self, endpoint: Endpoint) {
            if self.conflicts_existing(&endpoint) {
                return;
            }
            println!("Registered endpoint: {:?}", endpoint);
            self.endpoints.push(endpoint);
        }

        fn conflicts_existing(&self, endpoint: &Endpoint) -> bool {
            let existing_paths: HashSet<&String> =
                HashSet::from_iter(self.endpoints.iter().flat_map(|e| e.get_all_paths()));
            let endpoint_paths: HashSet<&String> = endpoint.get_all_paths();

            let conflicting_endpoints = existing_paths.intersection(&endpoint_paths);
            if conflicting_endpoints.count() > 0 {
                println!(
                    "{:?} conflicts with existing paths: {:?}",
                    endpoint, endpoint_paths
                );
                return true;
            }
            return false;
        }

        fn map_to_absolute(&self, location: &String) -> PathBuf {
            let path = Path::new(location);
            match path.is_absolute() {
                true => path.to_path_buf(),
                false => {
                    let cleaned_location = match location.starts_with("./") {
                        true => &location[2..],
                        false => &location,
                    };
                    let current_dir = std::env::current_dir().unwrap();
                    Path::new(&current_dir).join(cleaned_location)
                }
            }
        }
    }

    pub struct EndpointProvider {
        endpoints: Vec<Endpoint>,
        resource_handler: HashMap<String, Arc<ResourceHandler>>,
    }

    impl EndpointProvider {
        pub fn match_endpoint(&self, path: String, method: HttpMethod) -> Option<&Endpoint> {
            println!(
                "Called to resolve endpoint for path {} with method {:?}",
                path, method
            );
            return self.endpoints.iter().find(|e| match &e.endpoint_type {
                EndpointType::Assets(_) => {
                    return path.starts_with(&e.path);
                }
                _ => (e.path == path || e.aliases.contains(&path)) && e.methods.contains(&method),
            });
        }
        pub fn execute(&self, r: &ResourceEndpoint, request: &HttpRequest) -> String {
            let handler = self.resource_handler.get(&r.resource_handler_id).unwrap();
            return handler.handle(request);
        }
    }

    #[derive(Debug, Clone)]
    pub struct Endpoint {
        pub path: String,
        pub endpoint_type: EndpointType,
        aliases: Vec<String>,
        methods: Vec<HttpMethod>,
    }

    #[derive(Debug, Clone)]
    pub struct StaticEndpoint {
        pub asset_path: String,
    }

    #[derive(Debug, Clone)]
    pub struct AssetEndpoint {
        pub asset_base: String,
    }

    #[derive(Debug, Clone)]
    pub struct ResourceEndpoint {
        pub resource_handler_id: String,
    }

    impl Endpoint {
        pub fn asset(path: String, asset_base_path: String, aliases: Vec<String>) -> Endpoint {
            return Endpoint {
                endpoint_type: EndpointType::StaticAsset(StaticEndpoint {
                    asset_path: asset_base_path,
                }),
                path,
                aliases,
                methods: vec![HttpMethod::Get],
            };
        }

        pub fn get_all_paths(&self) -> HashSet<&String> {
            let mut all_paths = self.aliases.iter().collect::<HashSet<&String>>();
            let path_set: HashSet<&String> = [&self.path].iter().cloned().collect();
            all_paths.extend(path_set);
            all_paths
        }
    }

    #[derive(Debug, Clone)]
    pub enum EndpointType {
        StaticAsset(StaticEndpoint),
        Assets(AssetEndpoint),
        Resource(ResourceEndpoint),
    }
}

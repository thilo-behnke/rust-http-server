pub mod parser {
    use crate::types::types::{GeneralRequest, HttpMethod, HttpRequest, HttpVersion};
    use std::collections::HashMap;

    pub fn parse(request: &str) -> Result<HttpRequest, &str> {
        let request_split: Vec<&str> = request.split("\r\n").collect();
        match request_split.as_slice() {
            [] => Err("Empty request"),
            [general_line, headers @ ..] => {
                return match parse_general(general_line) {
                    Ok(general) => Ok(HttpRequest {
                        general,
                        headers: parse_headers(headers),
                    }),
                    Err(_) => Err("Failed to parse request"),
                }
            }
        }
    }

    fn parse_headers(_headers: &[&str]) -> HashMap<String, String> {
        HashMap::new()
    }

    fn parse_general(general: &str) -> Result<GeneralRequest, &str> {
        fn match_method(method: &str) -> Result<HttpMethod, &str> {
            return match method.to_uppercase().as_str() {
                "GET" => Ok(HttpMethod::Get),
                "HEAD" => Ok(HttpMethod::Head),
                "OPTIONS" => Ok(HttpMethod::Options),
                "POST" => Ok(HttpMethod::Post),
                "PUT" => Ok(HttpMethod::Put),
                "DELETE" => Ok(HttpMethod::Delete),
                _ => Err("Unknown method type"),
            };
        }

        fn match_version(version: &str) -> Result<HttpVersion, &str> {
            return match version.to_lowercase().as_str() {
                "http/1.1" => Ok(HttpVersion::One),
                "http/2" => Ok(HttpVersion::Two),
                "http/3" => Ok(HttpVersion::Three),
                _ => Err("Unknown method type"),
            };
        }

        let general_split: Vec<&str> = general.split(" ").map(|word| word.trim()).collect();
        return match general_split.as_slice() {
            [method, path] => {
                return match match_method(method) {
                    Ok(m) => Ok(GeneralRequest {
                        method: m,
                        path,
                        version: HttpVersion::One,
                    }),
                    Err(e) => Err(e),
                }
            }
            // TODO: Refactor to use error handling for request method (and version)
            [method, path, version] => Ok(GeneralRequest {
                method: match_method(method).expect("valid method"),
                path,
                version: match_version(version).expect("valid version"),
            }),
            [] => Err("No general request information"),
            _ => Err("Invalid general request information"),
        };
    }
}

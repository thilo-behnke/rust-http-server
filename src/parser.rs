pub mod parser {
    use std::fmt;
    use std::collections::HashMap;
    use std::fmt::Formatter;

    pub fn parse(request: &str) -> Result<HttpRequest, &str> {
        let request_split: Vec<&str> = request.split("\r\n").collect();
        match request_split.as_slice() {
            [] => Err("Empty request"),
            [general_line, headers @ ..] => {
                let general = parse_general(general_line).expect("valid general request");
                return Ok(HttpRequest {general, headers: HashMap::new()})
            }
        }
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
                _ => Err("Unknown method type")
            };
        }

        fn match_version(version: &str) -> Result<HttpVersion, &str> {
            return match version.to_lowercase().as_str() {
                "http/1.1" => Ok(HttpVersion::One),
                "http/2" => Ok(HttpVersion::Two),
                "http/3" => Ok(HttpVersion::Three),
                _ => Err("Unknown method type")
            };
        }

        let general_split: Vec<&str> = general.split(" ").map(|word| word.trim()).collect();
        return match general_split.as_slice() {
            [ method, path ] => Ok(GeneralRequest {method: match_method(method).expect("valid method"), path, version: HttpVersion::One}),
            [ method, path, version ] => Ok(GeneralRequest {method: match_method(method).expect("valid method"), path, version: match_version(version).expect("valid version")}),
            [] => Err("No general request information"),
            _ => Err("Invalid general request information")
        };
    }

    pub struct GeneralRequest<'a> {
        method: HttpMethod,
        path: &'a str,
        version: HttpVersion
    }

    pub struct HttpRequest<'a> {
        general: GeneralRequest<'a>,
        headers: HashMap<&'a str, &'a str>
    }

    pub enum HttpMethod {
        Head, Options, Get, Post, Put, Delete
    }

    pub enum HttpVersion {
        One, Two, Three
    }

    impl fmt::Display for HttpMethod {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                HttpMethod::Get => write!(f, "GET"),
                HttpMethod::Head => write!(f, "HEAD"),
                HttpMethod::Options => write!(f, "OPTIONS"),
                HttpMethod::Post => write!(f, "POST"),
                HttpMethod::Put => write!(f, "PUT"),
                HttpMethod::Delete => write!(f, "DELETE")
            }
        }
    }

    impl fmt::Display for HttpVersion {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                HttpVersion::One => write!(f, "Http/1.1"),
                HttpVersion::Two => write!(f, "Http/2"),
                HttpVersion::Three => write!(f, "Http/3"),
            }
        }
    }

    impl fmt::Display for HttpRequest<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "HttpRequest [method=\"{}\", path=\"{}\", version=\"{}\"]", self.general.method, self.general.path, self.general.version)
        }
    }
}

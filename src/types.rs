pub mod types {
    use std::collections::HashMap;
    use std::fmt;
    use std::fmt::Formatter;

    pub struct GeneralRequest<'a> {
        pub method: HttpMethod,
        pub path: &'a str,
        pub version: HttpVersion,
    }

    pub struct HttpRequest<'a> {
        pub general: GeneralRequest<'a>,
        pub headers: HashMap<String, String>,
    }

    pub enum HttpMethod {
        Head,
        Options,
        Get,
        Post,
        Put,
        Delete,
    }

    pub enum HttpVersion {
        One,
        Two,
        Three,
    }

    impl fmt::Display for HttpMethod {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                HttpMethod::Get => write!(f, "GET"),
                HttpMethod::Head => write!(f, "HEAD"),
                HttpMethod::Options => write!(f, "OPTIONS"),
                HttpMethod::Post => write!(f, "POST"),
                HttpMethod::Put => write!(f, "PUT"),
                HttpMethod::Delete => write!(f, "DELETE"),
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
            write!(
                f,
                "HttpRequest [method=\"{}\", path=\"{}\", version=\"{}\"]",
                self.general.method, self.general.path, self.general.version
            )
        }
    }
}

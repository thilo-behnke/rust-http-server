pub mod request_helper {
    pub fn get_parameters_from_path(path: &str) -> Vec<RequestParameter> {
        let query_params = get_query_params(path);
        return query_params;
    }

    fn get_query_params(path: &str) -> Vec<RequestParameter> {
        if !path.contains("?") {
            return vec![];
        }
        let mut path_split = path.split("?");
        let query_str = path_split.nth(1);
        if let None = query_str {
            return vec![];
        }
        match query_str {
            None => vec![],
            Some(params) => {
                return params
                    .split("&")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|it| {
                        let params_split: Vec<&str> = it.split("=").collect();
                        let (name, value) =
                            (params_split[0], params_split[1]);
                        return RequestParameter::Query(RequestParameterValue { name, value });
                    })
                    .collect()
            }
        }
    }

    pub fn clean_path(path: &str) -> &str {
        match path.split("?").nth(0) {
            Some(val) => val,
            None => path,
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum RequestParameter<'a> {
        Query(RequestParameterValue<'a>),
        Path(RequestParameterValue<'a>),
    }

    #[derive(Debug, Copy, Clone)]
    pub struct RequestParameterValue<'a> {
        pub name: &'a str,
        pub value: &'a str,
    }
}

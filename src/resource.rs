pub mod resource {
    use crate::request_helper::request_helper::get_parameters_from_path;
    use crate::types::types::HttpRequest;
    use std::sync::Arc;

    pub struct ResourceHandler {
        parameters: Vec<ResourceParameter>,
        handler: fn() -> String,
    }

    impl ResourceHandler {
        pub fn new(handler: fn() -> String, parameters: Vec<ResourceParameter>) -> ResourceHandler {
            ResourceHandler {
                parameters,
                handler,
            }
        }

        pub fn handle(&self, request: &HttpRequest) -> String {
            let path = request.general.path;
            return (self.handler)();
        }
    }

    pub struct ResourceParameter {
        name: String,
        l_type: ResourceParameterLocation,
        p_type: ResourceParameterType,
    }

    pub enum ResourceParameterLocation {
        Path,
        Query,
    }

    // TODO: Better way to do this in rust?
    pub enum ResourceParameterType {
        String,
        I8,
    }

    impl ResourceParameter {
        pub fn p_string(name: String, l_type: ResourceParameterLocation) -> ResourceParameter {
            return ResourceParameter {
                name,
                l_type,
                p_type: ResourceParameterType::String,
            };
        }

        pub fn p_i8(name: String, l_type: ResourceParameterLocation) -> ResourceParameter {
            return ResourceParameter {
                name,
                l_type,
                p_type: ResourceParameterType::I8,
            };
        }
    }
}

pub mod resource {
    use crate::types::types::HttpRequest;

    pub struct ResourceHandler {
        parameters: Vec<ResourceParameter>,
        handler: Box<dyn Fn() -> String + Sync + Send>,
    }

    impl ResourceHandler {
        pub fn new(handler: Box<dyn Fn() -> String + Sync + Send>, parameters: Vec<ResourceParameter>) -> ResourceHandler {
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

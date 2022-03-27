pub mod resource {
    use std::collections::HashMap;
    use crate::request_helper::request_helper::{RequestArgs, RequestArgValue};
    use crate::request_helper::request_helper::RequestArgs::{Path, Query};
    use crate::types::types::HttpRequest;

    pub struct ResourceHandler {
        parameters: Vec<ResourceParameter>,
        handler: Box<dyn Fn(&HashMap<&str, &RequestArgValue>) -> String + Sync + Send>,
    }

    impl ResourceHandler {
        pub fn new(handler: Box<dyn Fn(&HashMap<&str, &RequestArgValue>) -> String + Sync + Send>, parameters: Vec<ResourceParameter>) -> ResourceHandler {
            ResourceHandler {
                parameters,
                handler,
            }
        }

        pub fn handle(&self, request: &HttpRequest) -> String {
            let accepted_args = &request.general.args.iter().filter(|it| return match it {
                Query(arg) => {
                    let RequestArgValue { name, ..} = arg;
                    self.parameters.iter().any(|p| &p.name == name && p.l_type == ResourceParameterLocation::Query)
                },
                Path(arg) => {
                    let RequestArgValue { name, ..} = arg;
                    self.parameters.iter().any(|p| &p.name == name && p.l_type == ResourceParameterLocation::Path)
                },
            }).map(|it| return match it {
                Query(arg) => (arg.name, arg),
                Path(arg) => (arg.name, arg),
            }).collect();
            println!("Accepted args: {:?} vs all requested: {:?}", accepted_args, &request.general.args);
            return (self.handler)(&accepted_args);
        }
    }

    pub struct ResourceParameter {
        name: String,
        l_type: ResourceParameterLocation,
        p_type: ResourceParameterType,
    }

    #[derive(PartialEq)]
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

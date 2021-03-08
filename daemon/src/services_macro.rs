// A macro that generates various data structures and functions
// that operate on the list of available services.

macro_rules! declare_services {

    ( $( $feature:literal;$crate_name:ident;$service:ident ),* ) => {
        $(
            #[cfg(feature = $feature)]
            use $crate_name::service::$service;
        )*

        pub enum SharedStateKind {
            $(
                #[cfg(feature = $feature)]
                $service(Shared<<$service as Service<$service>>::State>),
            )*
        }

        impl SharedStateKind {
            pub fn is_locked(&self) -> bool {
                match &*self {
                    $(
                        #[cfg(feature = $feature)]
                        SharedStateKind::$service(shared) => shared.is_locked(),
                    )*
                }
            }

            pub fn log(&self) {
                use common::traits::StateLogger;

                match &*self {
                    $(
                        #[cfg(feature = $feature)]
                        SharedStateKind::$service(shared) => shared.lock().log(),
                    )*
                }
            }
        }

        pub type SharedStateMap = Shared<HashMap<String, SharedStateKind>>;

        pub fn create_shared_state() -> SharedStateMap {
            // The shared state for each service.
            let mut map = HashMap::new();
            $(
                #[cfg(feature = $feature)]
                map.insert(
                    $crate_name::generated::service::SERVICE_NAME.to_owned(),
                    SharedStateKind::$service($service::shared_state()),
                );

            )*
            Shared::adopt(map)
        }

        // The session only tracks services, not individual
        // objects.
        pub enum TrackableServices {
            $(
                #[cfg(feature = $feature)]
                $service(RefCell<$service>),
            )*
            Remote(RefCell<RemoteService>),
        }

        pub fn enabled_services(config: &Config, registrar: &RemoteServicesRegistrar) -> HashSet<String> {
            let mut services = HashSet::new();

            for (name, _id) in &registrar.services {
                if let Ok(lists) = fs::read_to_string(format!(
                    "{}/{}/valid_build_props.txt",
                    config.general.remote_services_path, name
                )) {
                    // Do not enable the remote service,
                    // if it defined a prop white list and the system do no match.
                    if !check_system_state(true, Some(&lists)).unwrap_or(false) {
                        continue;
                    }
                }
                services.insert(format!("{}:remote", name));
            }

            $(
                #[cfg(feature = $feature)]
                services.insert($crate_name::generated::service::SERVICE_NAME.to_owned());

            )*
            services
        }


        // Helper for Session::on_release_object
        pub fn on_release_object_helper(input: &Option<&TrackableServices>,
                                        req: &ReleaseObjectRequest,
                                        message: &mut BaseMessage) -> Result<bool, String> {
            match input {
                Some(obj) => match *obj {
                    $(
                        #[cfg(feature = $feature)]
                        TrackableServices::$service(ref service) => {
                            Ok(service.borrow_mut().release_object(req.object))
                        }
                    )*
                    TrackableServices::Remote(ref service) => {
                        Ok(service.borrow_mut().release_object(req.object))
                    }
                },
                None => {
                    Err(format!(
                        "Unable to find service with id: {}",
                        message.service))
                }
            }
        }

        // Helper for Session::process_base_message
        pub fn process_base_message_helper(input: &Option<&TrackableServices>,
                                           session_helper: &SessionSupport,
                                           message: &mut BaseMessage) -> Result<(), String> {
            match input {
                Some(obj) => match *obj {
                    $(
                    #[cfg(feature = $feature)]
                    TrackableServices::$service(ref service) => {
                        Ok(service.borrow_mut().on_request(session_helper, message))
                    }
                    )*
                    TrackableServices::Remote(ref service) => {
                        Ok(service.borrow_mut().on_request(session_helper, message))
                    }
                },
                None => {
                    Err(format!(
                    "Unable to find service with id: {}",
                    message.service))
                }
            }
        }

        // Helper for Session::on_create_service
        pub fn on_create_service_helper(session: &mut Session, s_id: TrackerId, req: &GetServiceRequest)-> CoreResponse {
            $(
                #[cfg(feature = $feature)]
                if req.name == $crate_name::generated::service::SERVICE_NAME {
                    if req.fingerprint != $crate_name::generated::service::SERVICE_FINGERPRINT {
                        error!("Fingerprint mismatch for service {}. Expected {} but got {}",
                               req.name, $crate_name::generated::service::SERVICE_FINGERPRINT, req.fingerprint);
                               return CoreResponse::GetService(GetServiceResponse::FingerprintMismatch);
                    }
                    let lock = session.shared_state.lock();
                    let state = match lock.get($crate_name::generated::service::SERVICE_NAME) {
                        Some(SharedStateKind::$service(data)) => data,
                        _ => panic!("Missing shared state for {}!!", $crate_name::generated::service::SERVICE_NAME),
                    };

                    let helpers = session
                        .session_helper
                        .new_with_session(SessionTrackerId::from(session.session_id, s_id));

                    let origin_attributes = session.origin_attributes.clone().unwrap();

                    if !$crate_name::generated::service::check_service_permission(&origin_attributes) {
                        error!(
                            "Could not create service {}: required permission not present.",
                            $crate_name::generated::service::SERVICE_NAME
                        );
                        return CoreResponse::GetService(GetServiceResponse::MissingPermission);
                    } else {
                        match $service::create(
                        &origin_attributes,
                        session.context.clone(),
                        state.clone(),
                        helpers,
                        ) {
                            Ok(s) => {
                                let s_item = TrackableServices::$service(RefCell::new(s));
                                let id = session.tracker.track(s_item);
                                return CoreResponse::GetService(GetServiceResponse::Success(id));
                            },
                            Err(err) => {
                                error!(
                                    "Could not create service {} !",
                                    $crate_name::generated::service::SERVICE_NAME
                                );
                                return CoreResponse::GetService(GetServiceResponse::InternalError(err));
                            }
                        }
                    }
                }
            )*

            CoreResponse::GetService(GetServiceResponse::UnknownService)
        }

        // Helper for Session::format_request
        pub fn format_request_helper(input: &Option<&TrackableServices>, session_helper: &SessionSupport, msg: &BaseMessage) -> String {
            match input {
                Some(obj) => match *obj {
                    $(
                        #[cfg(feature = $feature)]
                        TrackableServices::$service(ref service) => service
                        .borrow_mut()
                        .format_request(session_helper, &msg),
                    )*
                    TrackableServices::Remote(ref service) => service
                        .borrow_mut()
                        .format_request(session_helper, &msg),
                },
                None => format!("Unable to find service with id: {}", msg.service),
            }
        }

    };
}

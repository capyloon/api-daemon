/// Implementation of the test service.
use crate::generated::common::*;
use crate::generated::service::*;
use crate::private_traits::PrivateTestTrait;
use common::core::BaseMessage;
use common::object_tracker::ObjectTracker;
use common::traits::{
    CommonResponder, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport, Shared,
    SharedSessionContext, SimpleObjectTracker, StateLogger, TrackerId,
};
use common::{JsonValue, SystemTime};
use log::{error, info};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct FooProviderImpl {
    id: TrackerId,
    amount: f64,
    event_dispatcher: FooProviderEventDispatcher,
}

impl FooProviderMethods for FooProviderImpl {
    fn do_it(&mut self, responder: &FooProviderDoItResponder, what: String) {
        info!("FooProviderImpl::do_it `{}`", what);
        let len = what.len();
        self.event_dispatcher.dispatch_signal(what);
        responder.resolve(len as _);
    }

    fn get_amount(&mut self, responder: &FooProviderGetAmountResponder) {
        responder.resolve(self.amount);
    }

    fn set_amount(&mut self, value: f64) {
        self.amount = value;
    }
}

impl SimpleObjectTracker for FooProviderImpl {
    fn id(&self) -> TrackerId {
        self.id
    }
}

pub struct SharedFooProviderImpl {
    id: TrackerId,
    amount: f64,
}

impl SharedFooProviderMethods for SharedFooProviderImpl {
    fn do_it(&mut self, responder: &SharedFooProviderDoItResponder, what: String) {
        info!("SharedFooProviderImpl::do_it `{}`", what);
        let len = what.len();
        responder.resolve(len as _);
    }

    fn get_amount(&mut self, responder: &SharedFooProviderGetAmountResponder) {
        responder.resolve(self.amount);
    }

    fn set_amount(&mut self, value: f64) {
        self.amount = value;
    }
}

impl SimpleObjectTracker for SharedFooProviderImpl {
    fn id(&self) -> TrackerId {
        self.id
    }
}

pub struct SharedCustomProviderImpl {
    id: TrackerId,
    amount: f64,
}

impl SharedCustomProviderMethods for SharedCustomProviderImpl {
    fn do_it(&mut self, responder: &SharedCustomProviderDoItResponder, what: String) {
        info!("SharedFooProviderImpl::do_it `{}`", what);
        let len = what.len();
        responder.resolve(len as _);
    }

    fn get_amount(&mut self, responder: &SharedCustomProviderGetAmountResponder) {
        responder.resolve(self.amount);
    }

    fn set_amount(&mut self, value: f64) {
        self.amount = value;
    }
}

impl PrivateTestTrait for SharedCustomProviderImpl {
    fn hello_world(&self) {
        println!("Hello World!");
    }
}

impl SimpleObjectTracker for SharedCustomProviderImpl {
    fn id(&self) -> TrackerId {
        self.id
    }
}

pub struct TestSharedData {
    request_count: u32,
}

impl StateLogger for TestSharedData {}

pub struct TestServiceImpl {
    id: TrackerId,
    event_dispatcher: TestFactoryEventDispatcher,
    tracker: Arc<Mutex<TestServiceTrackerType>>,
    proxy_tracker: TestServiceProxyTracker,
    state_prop: bool,
    state: Shared<TestSharedData>,
    helper: SessionSupport,
}

impl TestService for TestServiceImpl {
    fn get_tracker(&mut self) -> Arc<Mutex<TestServiceTrackerType>> {
        self.tracker.clone()
    }

    fn get_proxy_tracker(&mut self) -> &mut TestServiceProxyTracker {
        &mut self.proxy_tracker
    }
}

impl TestFactoryMethods for TestServiceImpl {
    fn crash(&mut self, responder: &TestFactoryCrashResponder) {
        // self.sigsegv();
        responder.resolve();
    }

    fn postpone(&mut self, responder: &TestFactoryPostponeResponder, timeout: i64) {
        info!("postpone {}", timeout);

        // Make the responder and event dispatcher available in the thread.
        let responder = responder.clone();
        let event_dispatcher = self.event_dispatcher.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timeout as u64));
            responder.resolve(true);

            event_dispatcher.dispatch_timeout(TimeoutEvent {
                status: true,
                things: BagOfThings {
                    one: 1,
                    two: "two".into(),
                },
            });
        });
    }

    fn update_bag(
        &mut self,
        responder: &TestFactoryUpdateBagResponder,
        _index: i64,
        _bag: BagOfThings,
    ) {
        info!("update_bag");
        responder.reject();
    }

    fn default_bag(&mut self, responder: &TestFactoryDefaultBagResponder) {
        info!("default_bag");
        responder.reject("Something went wrong!".into());
    }

    fn zero_or_more_bags(&mut self, responder: &TestFactoryZeroOrMoreBagsResponder, zero: bool) {
        info!("zero_or_more_bag");
        if zero {
            responder.resolve(None);
        } else {
            let mut list = Vec::<BagOfThings>::new();
            list.push(BagOfThings {
                one: 1,
                two: "a".to_string(),
            });
            list.push(BagOfThings {
                one: 2,
                two: "b".to_string(),
            });
            list.push(BagOfThings {
                one: 3,
                two: "c".to_string(),
            });
            responder.resolve(Some(list));
        }
    }

    fn one_or_more_bags(&mut self, responder: &TestFactoryOneOrMoreBagsResponder, one: bool) {
        info!("one_or_more_bag");
        let mut list = Vec::<BagOfThings>::new();
        if one {
            list.push(BagOfThings {
                one: 1,
                two: "a".to_string(),
            });
        } else {
            list.push(BagOfThings {
                one: 1,
                two: "a".to_string(),
            });
            list.push(BagOfThings {
                one: 2,
                two: "b".to_string(),
            });
            list.push(BagOfThings {
                one: 3,
                two: "c".to_string(),
            });
        }
        responder.resolve(list);
    }

    fn get_state(&mut self, responder: &TestFactoryGetStateResponder) {
        info!("get_state");
        responder.resolve(self.state_prop);
    }

    fn set_state(&mut self, value: bool) {
        info!("set_state");
        self.state_prop = value;
    }

    fn get_blob(&mut self, responder: &TestFactoryGetBlobResponder, size: i64) {
        info!("get_blob {}", size);
        let mut result = Vec::new();
        result.resize(size as usize, 42);
        responder.resolve(result);
    }

    fn echo_json(&mut self, responder: &TestFactoryEchoJsonResponder, input: JsonValue) {
        info!("echo_json");
        responder.resolve(input);
    }

    fn get_provider(&mut self, responder: &TestFactoryGetProviderResponder) {
        info!("get_provider");
        let mut tracker = self.tracker.lock();
        let id = tracker.next_id();
        let event_dispatcher = FooProviderEventDispatcher::from(self.helper.clone(), id);
        let provider = Rc::new(FooProviderImpl {
            id,
            event_dispatcher,
            amount: 0.0,
        });
        tracker.track(TestServiceTrackedObject::FooProvider(provider.clone()));
        responder.resolve(provider);
    }

    fn get_shared_provider(&mut self, responder: &TestFactoryGetSharedProviderResponder) {
        info!("get_shared_provider");
        let mut tracker = self.tracker.lock();
        let id = tracker.next_id();
        let provider = Arc::new(Mutex::new(SharedFooProviderImpl { id, amount: 0.0 }));
        tracker.track(TestServiceTrackedObject::SharedFooProvider(
            provider.clone(),
        ));
        responder.resolve(provider);
    }

    fn get_shared_custom_provider(
        &mut self,
        responder: &TestFactoryGetSharedCustomProviderResponder,
    ) {
        info!("get_shared_custom_provider");
        let mut tracker = self.tracker.lock();
        let id = tracker.next_id();
        let provider = Arc::new(Mutex::new(SharedCustomProviderImpl { id, amount: 0.0 }));
        tracker.track(TestServiceTrackedObject::SharedCustomProvider(
            provider.clone(),
        ));
        responder.resolve(provider);
    }

    fn test_string_arrays(
        &mut self,
        responder: &TestFactoryTestStringArraysResponder,
        input: Vec<String>,
    ) {
        info!("test_string_arrays with {} strings", input.len());
        responder.resolve(input.len() as _);
    }

    fn optional(&mut self, responder: &TestFactoryOptionalResponder, optional: bool) {
        if optional {
            responder.resolve(Some(42));
        } else {
            responder.resolve(None)
        }
    }

    fn one_or_more(&mut self, responder: &TestFactoryOneOrMoreResponder, one: bool) {
        if one {
            responder.resolve(vec![42]);
        } else {
            responder.resolve(vec![42, 32, 22]);
        }
    }

    fn zero_or_more(&mut self, responder: &TestFactoryZeroOrMoreResponder, zero: bool) {
        if zero {
            responder.resolve(None);
        } else {
            responder.resolve(Some(vec![42, 32, 22]));
        }
    }

    fn arity_dict(
        &mut self,
        responder: &TestFactoryArityDictResponder,
        optional: bool,
        zero: bool,
        one: bool,
    ) {
        let dict = ArityDict {
            optional: if optional { Some(42) } else { None },
            zero_or_more: if zero { None } else { Some(vec![42, 32, 22]) },
            one_or_more: if one { vec![42] } else { vec![42, 32, 22] },
            zero_or_more_bags: if zero {
                None
            } else {
                Some(vec![
                    BagOfThings {
                        one: 1,
                        two: "a".to_string(),
                    },
                    BagOfThings {
                        one: 2,
                        two: "b".to_string(),
                    },
                    BagOfThings {
                        one: 3,
                        two: "c".to_string(),
                    },
                ])
            },
            one_or_more_bags: if one {
                vec![BagOfThings {
                    one: 1,
                    two: "a".to_string(),
                }]
            } else {
                vec![
                    BagOfThings {
                        one: 1,
                        two: "a".to_string(),
                    },
                    BagOfThings {
                        one: 2,
                        two: "b".to_string(),
                    },
                    BagOfThings {
                        one: 3,
                        two: "c".to_string(),
                    },
                ]
            },
            enums: if one {
                vec![Possibilities::One]
            } else {
                vec![Possibilities::One, Possibilities::Two, Possibilities::Three]
            },
        };
        responder.resolve(dict);
    }

    fn echo_arg_optional(
        &mut self,
        responder: &TestFactoryEchoArgOptionalResponder,
        arg: Option<i64>,
    ) {
        responder.resolve(arg);
    }

    fn echo_arg_one_or_more(
        &mut self,
        responder: &TestFactoryEchoArgOneOrMoreResponder,
        arg: Vec<i64>,
    ) {
        responder.resolve(arg);
    }

    fn echo_arg_zero_or_more(
        &mut self,
        responder: &TestFactoryEchoArgZeroOrMoreResponder,
        arg: Option<Vec<i64>>,
    ) {
        responder.resolve(arg);
    }

    fn configure_option(
        &mut self,
        responder: &TestFactoryConfigureOptionResponder,
        option: ConfigureOptionDictionary,
    ) {
        let res = match option.enabled {
            Some(val) => format!("{}", val),
            None => "unknown".to_owned(),
        };
        responder.resolve(res);
    }

    fn add_observer(
        &mut self,
        responder: &TestFactoryAddObserverResponder,
        name: String,
        observer: ObjectRef,
    ) {
        info!("Adding observer {:?}", observer);

        // Call handle(name) on the observer after 1s.
        match self.proxy_tracker.get(&observer) {
            Some(TestServiceProxy::Callback(callback)) => {
                let mut callback = callback.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    let receiver = callback.handle(name);
                    match receiver.recv().unwrap() {
                        Ok(res) => info!("callback.handle() success, value is {}", res),
                        Err(_) => error!("callback.handle() errored."),
                    }
                });
            }
            _ => error!("Failed to get traked callback"),
        }

        responder.resolve();
    }

    fn remove_observer(
        &mut self,
        responder: &TestFactoryRemoveObserverResponder,
        _name: String,
        _observer: ObjectRef,
    ) {
        responder.resolve();
    }

    fn add_time(
        &mut self,
        responder: &TestFactoryAddTimeResponder,
        start: SystemTime,
        seconds: i64,
    ) {
        let res = start
            .checked_add(Duration::from_secs(seconds as _))
            .unwrap();
        responder.resolve(res.into());
    }

    fn generate_timeout_event(&mut self, responder: &TestFactoryGenerateTimeoutEventResponder) {
        responder.resolve(TimeoutEvent {
            status: true,
            things: BagOfThings {
                one: 1,
                two: "two".into(),
            },
        });
    }

    fn missing_permission(&mut self, responder: &TestFactoryMissingPermissionResponder) {
        responder.permission_error(
            "test-permission",
            "The missing_permission() function needs a permission!",
        );
    }

    fn echo_date(&mut self, responder: &TestFactoryEchoDateResponder, input: SystemTime) {
        responder.resolve(input);
    }

    fn echo_somethings(
        &mut self,
        responder: &TestFactoryEchoSomethingsResponder,
        input: SomeThings,
    ) {
        responder.resolve(input);
    }

    fn echo_morethings(
        &mut self,
        responder: &TestFactoryEchoMorethingsResponder,
        input: MoreThings,
    ) {
        responder.resolve(input);
    }
}

impl TestServiceImpl {
    #[allow(dead_code)]
    fn sigsegv(&mut self) {
        let ptr: *mut u32 = ptr::null_mut();
        unsafe {
            *ptr = 0;
        }
    }
}

impl Service<TestServiceImpl> for TestServiceImpl {
    // Shared among instances.
    type State = TestSharedData;

    fn shared_state() -> Shared<Self::State> {
        Shared::adopt(TestSharedData { request_count: 0 })
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        state: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Result<TestServiceImpl, String> {
        info!("TestService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher =
            TestFactoryEventDispatcher::from(helper.clone(), 0 /* object id */);
        Ok(TestServiceImpl {
            id: service_id,
            event_dispatcher,
            tracker: Arc::new(Mutex::new(ObjectTracker::default())),
            proxy_tracker: HashMap::new(),
            state,
            state_prop: true,
            helper,
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<TestServiceFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("TestService request: {:?}", req),
            Err(err) => format!("Unable to format TestService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        {
            let mut shared = self.state.lock();
            shared.request_count += 1;
            info!("TestService request count: {}", shared.request_count);
        }
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        self.proxy_tracker.remove(&object_id.into()).is_some()
    }
}

impl Drop for TestServiceImpl {
    fn drop(&mut self) {
        info!("Dropping Test Service #{}", self.id);
    }
}

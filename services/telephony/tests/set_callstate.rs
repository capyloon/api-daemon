use common::traits::{
    EmptyConfig, IdFactory, MessageSender, OriginAttributes, Service, SessionContext,
    SessionSupport, SessionTrackerId, Shared, SharedEventMap, SharedServiceState, StdSender,
};
use std::collections::HashSet;
use std::sync::mpsc;
use telephony_service::generated::common::*;
use telephony_service::service::TelephonyService;

#[test]
fn test_set_callstate() {
    TelephonyService::init_shared_state(&EmptyConfig);

    let permissions = HashSet::new();
    let attr = OriginAttributes::new("client-id-0", permissions);
    let context = Shared::adopt(SessionContext::default());
    let (sender, _receiver) = mpsc::channel();
    let id_factory = Shared::adopt(IdFactory::new(0));
    let event_map: SharedEventMap = Shared::default();

    let helpers = SessionSupport::new(
        SessionTrackerId::from(1, 1),
        MessageSender::new(Box::new(StdSender::new(&sender))),
        id_factory.clone(),
        event_map.clone(),
    );

    if let Ok(mut ts) = TelephonyService::create(&attr, context.clone(), helpers) {
        ts.set_call_state(CallState::Ringing);
    } else {
        panic!("new utils session failed!");
    }

    let helpers = SessionSupport::new(
        SessionTrackerId::from(2, 2),
        MessageSender::new(Box::new(StdSender::new(&sender))),
        id_factory.clone(),
        event_map.clone(),
    );

    if let Ok(_ts) = TelephonyService::create(&attr, context, helpers) {
        // the later service should get same value set by previoius service.
        assert_eq!(
            TelephonyService::shared_state().lock().call_state(),
            CallState::Ringing,
            "Call states must be equal!!!"
        );
    } else {
        panic!("new utils session failed!");
    }
}

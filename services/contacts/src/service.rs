/// Implementation of the contacts service.
use crate::cursor::ContactDbCursor;
use crate::db::ContactsDb;
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use common::object_tracker::ObjectTracker;
use common::traits::{
    CommonResponder, DispatcherId, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport,
    Shared, SharedSessionContext, SimpleObjectTracker, StateLogger, TrackerId,
};
use log::{debug, error, info};
use std::rc::Rc;
use std::thread;

pub struct ContactsSharedData {
    pub db: ContactsDb,
}

impl StateLogger for ContactsSharedData {
    fn log(&self) {
        self.db.log();
    }
}

// The struct used to implement the ContactCursor interface.
// It simply wraps a database cursor.
struct ContactCursorImpl {
    id: TrackerId,
    cursor: ContactDbCursor,
}

impl ContactCursorImpl {
    fn new(id: TrackerId, cursor: ContactDbCursor) -> Self {
        Self { id, cursor }
    }
}

impl SimpleObjectTracker for ContactCursorImpl {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl ContactCursorMethods for ContactCursorImpl {
    fn next(&mut self, responder: &ContactCursorNextResponder) {
        match self.cursor.next() {
            Some(contacts) => responder.resolve(contacts),
            None => responder.reject(),
        }
    }
}

lazy_static! {
    pub(crate) static ref CONTACTS_SHARED_DATA: Shared<ContactsSharedData> =
        Shared::adopt(ContactsSharedData {
            db: ContactsDb::new(ContactsFactoryEventBroadcaster::default())
        });
}

pub struct ContactsService {
    id: TrackerId,
    state: Shared<ContactsSharedData>,
    dispatcher_id: DispatcherId,
    tracker: ContactsManagerTrackerType,
    origin_attributes: OriginAttributes,
}

impl ContactsManager for ContactsService {
    fn get_tracker(&mut self) -> &mut ContactsManagerTrackerType {
        &mut self.tracker
    }
}

impl ContactsFactoryMethods for ContactsService {
    fn clear_contacts(&mut self, responder: &ContactsFactoryClearContactsResponder) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "clear contacts",
        ) {
            return;
        }

        match self.state.lock().db.clear_contacts() {
            Ok(()) => responder.resolve(),
            Err(err) => {
                debug!("clear error is {}", err);
                responder.reject()
            }
        }
    }

    fn get(&mut self, responder: &ContactsFactoryGetResponder, id: String, only_main_data: bool) {
        let responder = responder.clone();
        let shared = self.state.clone();
        thread::spawn(move || {
            let db = &shared.lock().db;
            match db.get(&id, only_main_data) {
                Ok(value) => responder.resolve(value),
                Err(err) => {
                    error!("ContactsService::get error: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn add(&mut self, responder: &ContactsFactoryAddResponder, contacts: Vec<ContactInfo>) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:create",
            "add contacts",
        ) {
            return;
        }

        let responder = responder.clone();
        let shared = self.state.clone();
        thread::spawn(move || {
            let db = &mut shared.lock().db;
            match db.save(&contacts, false) {
                Ok(_) => responder.resolve(),
                Err(err) => {
                    error!("ContactsService::add error: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn update(&mut self, responder: &ContactsFactoryUpdateResponder, contacts: Vec<ContactInfo>) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "update contacts",
        ) {
            return;
        }

        let responder = responder.clone();
        let shared = self.state.clone();
        thread::spawn(move || {
            let db = &mut shared.lock().db;
            match db.save(&contacts, true) {
                Ok(_) => responder.resolve(),
                Err(err) => {
                    error!("ContactsService::update error: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn remove(&mut self, responder: &ContactsFactoryRemoveResponder, contact_ids: Vec<String>) {
        debug!("remove contacts");
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "remove contacts",
        ) {
            return;
        }

        match self.state.lock().db.remove(&contact_ids) {
            Ok(()) => {
                debug!("remove Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("remove Err:{}", err);
                responder.reject()
            }
        }
    }

    fn get_all(
        &mut self,
        responder: &ContactsFactoryGetAllResponder,
        options: ContactSortOptions,
        batch_size: i64,
        only_main_data: bool,
    ) {
        debug!(
            "service get_all called batch size is {}, only_main_data {}",
            batch_size, only_main_data
        );
        let state = self.state.lock();
        if let Some(db_cursor) = state.db.get_all(options, batch_size, only_main_data) {
            let id = self.tracker.next_id();
            let cursor = Rc::new(ContactCursorImpl::new(id, db_cursor));
            self.tracker
                .track(ContactsManagerTrackedObject::ContactCursor(cursor.clone()));
            responder.resolve(cursor);
        } else {
            responder.reject();
        }
    }

    fn find(
        &mut self,
        responder: &ContactsFactoryFindResponder,
        params: ContactFindSortOptions,
        batch_size: i64,
    ) {
        let state = self.state.lock();
        if let Some(db_cursor) = state.db.find(params, batch_size) {
            let id = self.tracker.next_id();
            let cursor = Rc::new(ContactCursorImpl::new(id, db_cursor));
            self.tracker
                .track(ContactsManagerTrackedObject::ContactCursor(cursor.clone()));
            responder.resolve(cursor);
        } else {
            responder.reject();
        }
    }

    fn matches(
        &mut self,
        responder: &ContactsFactoryMatchesResponder,
        filter_by_option: FilterByOption,
        filter: FilterOption,
        value: String,
    ) {
        debug!(
            "matches filter_by_option: {:?}, filter_option: {:?}, value: {}",
            filter_by_option, filter, value
        );

        let options = ContactFindSortOptions {
            sort_by: SortOption::Name,
            sort_order: Order::Ascending,
            sort_language: "".into(),
            filter_value: value,
            filter_option: filter,
            filter_by: vec![filter_by_option],
            only_main_data: true,
        };

        if let Some(mut db_cursor) = self.state.lock().db.find(options, 1) {
            if let Some(info) = db_cursor.next() {
                debug!("matches info.len: {}", info.len());
                responder.resolve(!info.is_empty());
            } else {
                responder.resolve(false);
            }
        } else {
            responder.reject();
        }
    }

    fn set_ice(
        &mut self,
        responder: &ContactsFactorySetIceResponder,
        contact_id: String,
        position: i64,
    ) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "set ice",
        ) {
            return;
        }

        if position <= 0 {
            debug!("set_ice with invalid position:{}, reject", position);
            return responder.reject();
        }
        match self.state.lock().db.set_ice(&contact_id, position) {
            Ok(()) => responder.resolve(),
            Err(err) => {
                debug!("set_ice error:{}", err);
                responder.reject()
            }
        }
    }

    fn remove_ice(&mut self, responder: &ContactsFactoryRemoveIceResponder, contact_id: String) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "remove ice",
        ) {
            return;
        }

        match self.state.lock().db.remove_ice(&contact_id) {
            Ok(()) => responder.resolve(),
            Err(err) => {
                debug!("remove_ice error:{}", err);
                responder.reject()
            }
        }
    }

    fn get_all_ice(&mut self, responder: &ContactsFactoryGetAllIceResponder) {
        match self.state.lock().db.get_all_ice() {
            Ok(value) => {
                debug!("get_all_ice Ok {:#?}", value);
                responder.resolve(Some(value))
            }
            Err(err) => {
                debug!("get_all_ice error: {}", err);
                responder.reject();
            }
        }
    }

    fn get_count(&mut self, responder: &ContactsFactoryGetCountResponder) {
        let state = self.state.lock();
        let db = &state.db;
        match db.count() {
            Ok(count) => responder.resolve(count as _),
            Err(err) => {
                error!("ContactsService::count error: {}", err);
                responder.reject();
            }
        }
    }

    fn import_vcf(&mut self, responder: &ContactsFactoryImportVcfResponder, vcf: String) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:create",
            "import contacts from vcf",
        ) {
            return;
        }

        let responder = responder.clone();
        let shared = self.state.clone();
        thread::spawn(move || {
            let db = &mut shared.lock().db;
            match db.import_vcf(&vcf) {
                Ok(count) => responder.resolve(count as _),
                Err(err) => {
                    error!("ContactsService::import_vcf error: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn add_blocked_number(
        &mut self,
        responder: &ContactsFactoryAddBlockedNumberResponder,
        number: String,
    ) {
        debug!("add_blocked_number, number:{}", number);
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "add blocked number",
        ) {
            return;
        }

        match self.state.lock().db.add_blocked_number(&number) {
            Ok(()) => {
                debug!("add_blocked_number Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("add_blocked_number Err:{}", err);
                responder.reject()
            }
        }
    }

    fn remove_blocked_number(
        &mut self,
        responder: &ContactsFactoryRemoveBlockedNumberResponder,
        number: String,
    ) {
        debug!("remove_blocked_number, number:{}", number);
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "remove blocked number",
        ) {
            return;
        }

        match self.state.lock().db.remove_blocked_number(&number) {
            Ok(()) => {
                debug!("remove_blocked_number Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("remove_blocked_number Err:{}", err);
                responder.reject()
            }
        }
    }

    fn get_all_blocked_numbers(
        &mut self,
        responder: &ContactsFactoryGetAllBlockedNumbersResponder,
    ) {
        debug!("get_all_blocked_numbers()");
        match self.state.lock().db.get_all_blocked_numbers() {
            Ok(vec) => {
                debug!("get_all_blocked_numbers Ok");
                responder.resolve(Some(vec))
            }
            Err(err) => {
                debug!("get_all_blocked_numbers Err:{}", err);
                responder.reject()
            }
        }
    }

    fn find_blocked_numbers(
        &mut self,
        responder: &ContactsFactoryFindBlockedNumbersResponder,
        options: BlockedNumberFindOptions,
    ) {
        debug!("find_blocked_numbers() options:{:?}", options);
        match self.state.lock().db.find_blocked_numbers(options) {
            Ok(vec) => {
                debug!("find_blocked_numbers Ok");
                responder.resolve(Some(vec))
            }
            Err(err) => {
                debug!("find_blocked_numbers Err:{}", err);
                responder.reject()
            }
        }
    }

    fn get_speed_dials(&mut self, responder: &ContactsFactoryGetSpeedDialsResponder) {
        debug!("get_speed_dials");
        match self.state.lock().db.get_speed_dials() {
            Ok(vec) => {
                debug!("get_speed_dials Ok");
                responder.resolve(Some(vec))
            }
            Err(err) => {
                debug!("get_speed_dials Err:{}", err);
                responder.reject()
            }
        }
    }

    fn add_speed_dial(
        &mut self,
        responder: &ContactsFactoryAddSpeedDialResponder,
        dial_key: String,
        tel: String,
        contact_id: String,
    ) {
        debug!(
            "add_speed_dial, dial_key:{}, tel:{}, contact_id:{}",
            dial_key, tel, contact_id
        );
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "add speed dial",
        ) {
            return;
        }

        match self
            .state
            .lock()
            .db
            .add_speed_dial(&dial_key, &tel, &contact_id)
        {
            Ok(()) => {
                debug!("add_speed_dial Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("add_speed_dial Err:{}", err);
                responder.reject()
            }
        }
    }

    fn update_speed_dial(
        &mut self,
        responder: &ContactsFactoryUpdateSpeedDialResponder,
        dial_key: String,
        tel: String,
        contact_id: String,
    ) {
        debug!(
            "update_speed_dial, dial_key:{}, tel:{}, contact_id:{}",
            dial_key, tel, contact_id
        );
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "update speed dial",
        ) {
            return;
        }

        match self
            .state
            .lock()
            .db
            .update_speed_dial(&dial_key, &tel, &contact_id)
        {
            Ok(()) => {
                debug!("update_speed_dial Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("update_speed_dial Err:{}", err);
                responder.reject()
            }
        }
    }

    fn remove_speed_dial(
        &mut self,
        responder: &ContactsFactoryRemoveSpeedDialResponder,
        dial_key: String,
    ) {
        debug!("remove_speed_dial, dial_key:{}", dial_key);
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "remove speed dial",
        ) {
            return;
        }

        match self.state.lock().db.remove_speed_dial(&dial_key) {
            Ok(()) => {
                debug!("remove_speed_dial Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("remove_speed_dial Err:{}", err);
                responder.reject()
            }
        }
    }

    fn remove_group(&mut self, responder: &ContactsFactoryRemoveGroupResponder, id: String) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "remove group",
        ) {
            return;
        }

        match self.state.lock().db.remove_group(&id) {
            Ok(()) => {
                debug!("remove_group Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("remove_group Err:{}", err);
                responder.reject()
            }
        }
    }

    fn add_group(&mut self, responder: &ContactsFactoryAddGroupResponder, name: String) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "add group",
        ) {
            return;
        }

        debug!("add_group called ,name {}", name);
        match self.state.lock().db.add_group(&name) {
            Ok(()) => {
                debug!("add_group Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("add_group Err:{}", err);
                responder.reject()
            }
        }
    }

    fn update_group(
        &mut self,
        responder: &ContactsFactoryUpdateGroupResponder,
        id: String,
        name: String,
    ) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "contacts:write",
            "update group",
        ) {
            return;
        }

        match self.state.lock().db.update_group(&id, &name) {
            Ok(()) => {
                debug!("update_group Ok");
                responder.resolve()
            }
            Err(err) => {
                debug!("update_group Err:{}", err);
                responder.reject()
            }
        }
    }

    fn get_contactids_from_group(
        &mut self,
        responder: &ContactsFactoryGetContactidsFromGroupResponder,
        group_id: String,
    ) {
        match self.state.lock().db.get_contactids_from_group(&group_id) {
            Ok(value) => {
                debug!("get_contactids_from_group Ok");
                responder.resolve(Some(value))
            }
            Err(err) => {
                debug!("get_contactids_from_group error: {}", err);
                responder.reject()
            }
        }
    }

    fn get_all_groups(&mut self, responder: &ContactsFactoryGetAllGroupsResponder) {
        match self.state.lock().db.get_all_groups() {
            Ok(value) => {
                debug!("get_all_groups Ok");
                responder.resolve(Some(value))
            }
            Err(err) => {
                debug!("get_all_groups error: {}", err);
                responder.reject();
            }
        }
    }
}

impl Service<ContactsService> for ContactsService {
    // Shared among instances.
    type State = ContactsSharedData;

    fn shared_state() -> Shared<Self::State> {
        let shared = &*CONTACTS_SHARED_DATA;
        shared.clone()
    }

    fn create(
        attrs: &OriginAttributes,
        _context: SharedSessionContext,
        state: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Result<ContactsService, String> {
        info!("ContactsService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = ContactsFactoryEventDispatcher::from(helper, 0 /* object id */);
        let dispatcher_id = state.lock().db.add_dispatcher(&event_dispatcher);
        Ok(ContactsService {
            id: service_id,
            state,
            dispatcher_id,
            tracker: ObjectTracker::default(),
            origin_attributes: attrs.clone(),
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<ContactsManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("ContactsService request: {:?}", req),
            Err(err) => format!("Unable to format ContactsService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        self.tracker.untrack(object_id)
    }
}

impl Drop for ContactsService {
    fn drop(&mut self) {
        debug!("Dropping Contacts Service #{}", self.id);
        let db = &mut self.state.lock().db;
        db.remove_dispatcher(self.dispatcher_id);
        self.tracker.clear();
    }
}

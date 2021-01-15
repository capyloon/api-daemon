use crate::config::Config;
use crate::session::Session;
use common::core::{
    BaseMessage, CoreResponse, GetServiceRequest, GetServiceResponse, ReleaseObjectRequest,
};
use common::device_info::check_system_state;
use common::remote_service::RemoteService;
use common::remote_services_registrar::RemoteServicesRegistrar;
use common::traits::{
    ObjectTrackerMethods, Service, SessionSupport, SessionTrackerId, Shared, TrackerId,
};
use log::error;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;

// Each service is setup with "feature-name";crate_name;ServiceName

declare_services!(
    "apps-service";apps_service;AppsService,
    "audiovolumemanager-service";audiovolume_service;AudioVolume,
    "contacts-service";contacts_service;ContactsService,
    "devicecapability-service";devicecapability_service;DeviceCapabilityService,
    "geckobridge-service";geckobridge;GeckoBridgeService,
    "libsignal-service";libsignal_service;SignalService,
    "powermanager-service";powermanager_service;PowerManager,
    "procmanager-service";procmanager_service;ProcManagerService,
    "tcpsocket-service";tcpsocket_service;TcpSocketService,
    "time-service";time_service;Time,
    "settings-service";settings_service;SettingsService
);

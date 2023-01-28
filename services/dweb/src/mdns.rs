// mDNS discovery support.

use crate::generated::common::Peer;
use crate::service::{KnownPeer, State};
use common::traits::Shared;
use log::{error, info};
use searchlight::{
    broadcast::{BroadcasterBuilder, BroadcasterHandle, ServiceBuilder},
    discovery::{DiscoveryBuilder, DiscoveryEvent, DiscoveryHandle, Responder},
    dns::{op::DnsResponse, rr::RData},
    net::IpVersion,
};
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;

pub struct MdnsDiscovery {
    broadcaster: Option<BroadcasterHandle>,
    discovery: Option<DiscoveryHandle>,
    did: String,
    device_id: String,
    device_desc: String,
    state: Shared<State>,
}

const MDNS_DOMAIN: &str = "_capyloon._udp.local.";

fn to_empty_err<E: std::error::Error>(err: E) -> () {
    log::error!("mdns error: {}", err);
}

fn get_ipaddrs() -> Vec<IpAddr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .iter()
        .filter_map(|iface| {
            if iface.is_loopback() {
                return None;
            }
            Some(iface.ip())
        })
        .collect()
}

// Returns (map of txt records, port)
fn get_device_props(dns_packet: &DnsResponse) -> (BTreeMap<String, String>, u16) {
    let mut props = BTreeMap::new();
    let mut port = 0;

    dns_packet.additionals().iter().for_each(|record| {
        if let Some(RData::TXT(txt)) = record.data() {
            for prop in txt.iter() {
                let parts: Vec<String> = String::from_utf8_lossy(prop)
                    .split("=")
                    .map(|item| item.to_owned())
                    .collect();
                if parts.len() == 2 {
                    props.insert(parts[0].clone(), parts[1].clone());
                }
            }
        } else if let Some(RData::SRV(srv)) = record.data() {
            port = srv.port();
        }
    });

    (props, port)
}

fn get_device_id(dns_packet: &DnsResponse) -> Option<String> {
    dns_packet.additionals().iter().find_map(|record| {
        if let Some(RData::SRV(_)) = record.data() {
            let name = record.name().to_utf8();
            let name = name.strip_suffix(MDNS_DOMAIN).unwrap_or(&name);
            let name = name.strip_suffix('.').unwrap_or(&name);
            Some(name.to_string())
        } else {
            None
        }
    })
}

fn peer_from_responder(responder: &Arc<Responder>) -> Option<KnownPeer> {
    let (props, port) = get_device_props(&responder.last_response);
    let device_id = get_device_id(&responder.last_response)?;

    let peer = KnownPeer {
        peer: Peer {
            did: props.get("did").cloned()?,
            device_id,
            device_desc: props.get("desc").cloned()?,
        },
        is_local: true,
        endpoint: format!("{}:{}", responder.addr.ip(), port),
    };

    Some(peer)
}

impl MdnsDiscovery {
    pub fn active(&self) -> bool {
        self.broadcaster.is_some()
    }

    fn on_peer_found(responder: Arc<Responder>, state: Shared<State>) {
        info!("mdns: on_peer_found");
        if let Some(peer) = peer_from_responder(&responder) {
            state.lock().on_peer_found(peer);
        } else {
            error!("Failed to create peer from {:?}", responder);
        }
    }

    fn on_peer_lost(responder: Arc<Responder>, state: Shared<State>) {
        info!("mdns: on_peer_lost");

        if let Some(device_id) = get_device_id(&responder.last_response) {
            state.lock().on_peer_lost(&device_id);
        } else {
            error!("Failed to get device id");
        }
    }

    fn start_broadcaster(&mut self) -> Result<(), ()> {
        let mut service_builder = ServiceBuilder::new(MDNS_DOMAIN, &self.device_id, 4242)
            .map_err(to_empty_err)?
            .ttl(30)
            .add_txt(format!("desc={}", self.device_desc))
            .add_txt(format!("did={}", self.did));

        for adr in get_ipaddrs() {
            service_builder = service_builder.add_ip_address(adr);
        }

        let broadcaster = BroadcasterBuilder::new()
            .add_service(service_builder.build().map_err(to_empty_err)?)
            .build(IpVersion::Both)
            .map_err(to_empty_err)?
            .run_in_background();

        self.broadcaster = Some(broadcaster);
        Ok(())
    }
}

impl crate::DiscoveryMechanism for MdnsDiscovery {
    fn with_state(
        state: Shared<State>,
        did: &str,
        device_id: &str,
        device_desc: &str,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self {
            broadcaster: None,
            discovery: None,
            did: did.to_owned(),
            device_desc: device_desc.to_owned(),
            device_id: device_id.to_owned(),
            state,
        })
    }

    fn start(&mut self) -> Result<(), ()> {
        info!("mdns: start");

        self.start_broadcaster()?;

        // Setup Discovery.
        let (found_tx, found_rx) = std::sync::mpsc::sync_channel(0);

        let discovery = DiscoveryBuilder::new()
            .service(MDNS_DOMAIN)
            .map_err(to_empty_err)?
            .build(IpVersion::Both)
            .map_err(to_empty_err)?
            .run_in_background(move |event| {
                found_tx.try_send(event).ok();
            });
        self.discovery = Some(discovery);

        // Spawn a thread to listen to events.
        let state = self.state.clone();
        let _ = thread::Builder::new()
            .name("mdns events".into())
            .spawn(move || {
                loop {
                    if let Ok(event) = found_rx.recv() {
                        match event {
                            DiscoveryEvent::ResponderFound(responder) => {
                                Self::on_peer_found(responder, state.clone());
                            }
                            DiscoveryEvent::ResponderLost(responder) => {
                                Self::on_peer_lost(responder, state.clone());
                            }
                            _ => {
                                // Nothing to do for ResponseUpdate
                            }
                        }
                    } else {
                        break;
                    }
                }
                info!("mdns thread complete");
            });

        // Start the server that will be used for the offer/answer exchange.
        let server_state = self.state.clone();
        

        Ok(())
    }

    fn stop(&mut self) -> Result<(), ()> {
        info!("mdns: stop");

        // The drop implementation of the handles will call shutdown() for us.
        self.broadcaster = None;
        self.discovery = None;

        Ok(())
    }
}

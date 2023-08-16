use crate::structs::{TunnelNewClient, TunnelOpen, TunnelRequest};

#[allow(dead_code)]
enum Event {
    TunnelRequest(TunnelRequest),
    TunnelOpen(TunnelOpen),
    TunnelClose,
    TunnelNewClient(TunnelNewClient),
}

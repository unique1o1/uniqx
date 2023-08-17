use crate::structs::{NewClient, TunnelOpen, TunnelRequest};

#[allow(dead_code)]
enum Event {
    TunnelRequest(TunnelRequest),
    TunnelOpen(TunnelOpen),
    TunnelClose,
    TunnelNewClient(NewClient),
}

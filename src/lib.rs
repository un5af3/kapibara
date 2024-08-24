//! Kapibara Library
pub mod error;
pub use error::{DispatchError, DnsError, InboundError, OptionError, OutboundError, RouteError};

pub mod inbound;
pub use inbound::{Inbound, InboundOption};

pub mod outbound;
pub use outbound::{Outbound, OutboundOption};

pub mod dispatch;
pub use dispatch::{Dispatch, DispatchOption};

pub mod route;
pub use route::{Route, RouteOption, RouteRule, RouteRuleOption};

pub mod dns;
pub use dns::DnsOption;

pub mod codec;
pub use codec::Codec;

pub mod io;

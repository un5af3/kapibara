//! Kapibara Route

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{OptionError, RouteError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteOption {
    pub rules: Vec<RouteRuleOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteRuleOption {
    pub dns: bool,
    pub inbound: Vec<String>,
    pub outbound: String,
}

pub struct Route {
    pub in_to_out: HashMap<String, RouteRule>,
}

pub struct RouteRule {
    pub dns: bool,
    pub outbound: String,
}

impl Route {
    pub fn init(option: RouteOption) -> Result<Self, RouteError> {
        let mut in_to_out = HashMap::new();
        for rule in option.rules {
            for in_tag in rule.inbound {
                if let Some(other) = in_to_out.insert(
                    in_tag,
                    RouteRule {
                        outbound: rule.outbound.clone(),
                        dns: rule.dns,
                    },
                ) {
                    return Err(OptionError::DuplicateTag(other.outbound).into());
                }
            }
        }

        Ok(Self { in_to_out })
    }

    pub fn ask_inbound(&self, in_tag: &str) -> Option<&RouteRule> {
        self.in_to_out.get(in_tag)
    }
}

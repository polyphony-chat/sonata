use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Information sent to the server by a client, when the client wants to create a new account.
///
/// ## Important Note
///
/// sonata is in an MVP phase. As such, things like this `RegisterSchema` are subject to a lot of
/// change. If you build clients around sonata, expect things to break in future versions.
pub struct RegisterSchema {
    /// Whether the client has agreed to the terms of service offered by the instance.
    pub tos_consent: bool,
    /// The local name the client would like to choose
    pub local_name: String,
    /// A password for the clients' new account
    pub password: String,
    /// Optional: An invite code, which the client got referred to this instance with.
    pub invite: Option<String>,
}

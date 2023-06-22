#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_slice, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
};
use cw_storage_plus::Item;
use mesh_apis::ibc::{
    ack_success, validate_channel_order, AddValidator, AddValidatorsAck, ConsumerPacket,
    ProtocolVersion, RemoveValidatorsAck,
};

use crate::{
    crdt::{CrdtState, ValUpdate},
    error::ContractError,
    msg::AuthorizedEndpoint,
};

/// This is the maximum version of the Mesh Security protocol that we support
const SUPPORTED_IBC_PROTOCOL_VERSION: &str = "1.0.0";
/// This is the minimum version that we are compatible with
const MIN_IBC_PROTOCOL_VERSION: &str = "1.0.0";

// IBC specific state
pub const AUTH_ENDPOINT: Item<AuthorizedEndpoint> = Item::new("auth_endpoint");

// TODO: expected endpoint
pub const IBC_CHANNEL: Item<IbcChannel> = Item::new("ibc_channel");

pub const VAL_CRDT: CrdtState = CrdtState::new();

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    // ensure we have no channel yet
    if IBC_CHANNEL.may_load(deps.storage)?.is_some() {
        return Err(ContractError::IbcChannelAlreadyOpen);
    }
    // ensure we are called with OpenInit
    let (channel, counterparty_version) = match msg {
        IbcChannelOpenMsg::OpenInit { .. } => return Err(ContractError::IbcOpenInitDisallowed),
        IbcChannelOpenMsg::OpenTry {
            channel,
            counterparty_version,
        } => (channel, counterparty_version),
    };

    // verify the ordering is correct
    validate_channel_order(&channel.order)?;

    // assert expected endpoint
    let authorized = AUTH_ENDPOINT.load(deps.storage)?;
    if authorized.connection_id != channel.connection_id
        || authorized.port_id != channel.counterparty_endpoint.port_id
    {
        // FIXME: do we need a better error here?
        return Err(ContractError::Unauthorized);
    }

    // we handshake with the counterparty version, it must not be empty
    let v: ProtocolVersion = from_slice(counterparty_version.as_bytes())?;
    // if we can build a response to this, then it is compatible. And we use the highest version there
    let version = v.build_response(SUPPORTED_IBC_PROTOCOL_VERSION, MIN_IBC_PROTOCOL_VERSION)?;

    let response = Ibc3ChannelOpenResponse {
        version: version.to_string()?,
    };
    Ok(Some(response))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// once it's established, we store data
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // ensure we have no channel yet
    if IBC_CHANNEL.may_load(deps.storage)?.is_some() {
        return Err(ContractError::IbcChannelAlreadyOpen);
    }
    // ensure we are called with OpenConfirm
    let channel = match msg {
        IbcChannelConnectMsg::OpenConfirm { channel } => channel,
        IbcChannelConnectMsg::OpenAck { .. } => return Err(ContractError::IbcOpenInitDisallowed),
    };

    // Version negotiation over, we can only store the channel
    IBC_CHANNEL.save(deps.storage, &channel)?;

    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    todo!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
// this accepts validator sync packets and updates the crdt state
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    // There is only one channel, so we don't need to switch.
    // We also don't care about packet sequence as this is fully commutative.
    let packet: ConsumerPacket = from_slice(&msg.packet.data)?;
    let ack = match packet {
        ConsumerPacket::AddValidators(to_add) => {
            for AddValidator {
                valoper,
                pub_key,
                start_height,
                start_time,
            } in to_add
            {
                let update = ValUpdate {
                    pub_key,
                    start_height,
                    start_time,
                };
                VAL_CRDT.add_validator(deps.storage, &valoper, update)?;
            }
            ack_success(&AddValidatorsAck {})?
        }
        ConsumerPacket::RemoveValidators(to_remove) => {
            for valoper in to_remove {
                VAL_CRDT.remove_validator(deps.storage, &valoper)?;
            }
            ack_success(&RemoveValidatorsAck {})?
        }
    };

    // return empty success ack
    Ok(IbcReceiveResponse::new().set_ack(ack))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}
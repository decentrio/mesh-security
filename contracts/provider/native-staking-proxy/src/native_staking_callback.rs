use cosmwasm_std::{CustomMsg, Response, StdError};
use sylvia::types::ExecCtx;
use sylvia::{interface, schemars};

/// This defines the interfaces the native-staking-proxy contract can call on native-staking
#[interface]
pub trait NativeStakingCallback {
    type Error: From<StdError>;
    type ExecC: CustomMsg;
 
    /// This sends tokens back from the proxy to native-staking. (See info.funds)
    /// The native-staking contract can determine which user it belongs to via an internal Map.
    /// The native-staking contract will then send those tokens back to vault and release the claim.
    #[sv::msg(exec)]
    fn release_proxy_stake(&self, _ctx: ExecCtx) -> Result<Response<Self::ExecC>, Self::Error>;
}

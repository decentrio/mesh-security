use cosmwasm_std::{ensure_eq, DistributionMsg, Response, Uint128, VoteOption, WeightedVoteOption};
use cw2::set_contract_version;
use cw_storage_plus::Item;

use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, schemars};

use crate::error::ContractError;
use crate::types::{ClaimsResponse, Config, ConfigResponse};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct NativeStakingProxyContract<'a> {
    // TODO
    config: Item<'a, Config>,
}

#[contract(error=ContractError)]
impl NativeStakingProxyContract<'_> {
    pub const fn new() -> Self {
        Self {
            config: Item::new("config"),
        }
    }

    /// The caller of the instantiation will be the native-staking contract.
    /// We stake `funds.info` on the given validator
    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        denom: String,
        owner: String,
        validator: String,
    ) -> Result<Response, ContractError> {
        let config = Config {
            denom,
            parent: ctx.info.sender.clone(),
            owner: ctx.deps.api.addr_validate(&owner)?,
        };
        self.config.save(ctx.deps.storage, &config)?;
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // stake info.funds on validator
        let res = self.stake(ctx, validator)?;

        // set owner as recipient of future withdrawls
        let set_withdrawl = DistributionMsg::SetWithdrawAddress {
            address: config.parent.into_string(),
        };
        Ok(res.add_message(set_withdrawl))
    }

    /// stakes the tokens from `info.funds` to the given validator.
    /// can only be called by the parent contract.
    #[msg(exec)]
    fn stake(&self, ctx: ExecCtx, _validator: String) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.parent, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    /// restakes the given amount from the one validator to another on behalf of the calling user.
    /// returns an error if the user doesn't have such stake.
    #[msg(exec)]
    fn restake(
        &self,
        ctx: ExecCtx,
        _from_validator: String,
        _to_validator: String,
        _amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    /// Vote with the users stake (over all delegations)
    #[msg(exec)]
    fn vote(
        &self,
        ctx: ExecCtx,
        _proposal_id: String,
        _vote: VoteOption,
    ) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    /// Vote with the users stake (over all delegations)
    #[msg(exec)]
    fn vote_weighted(
        &self,
        ctx: ExecCtx,
        _proposal_id: String,
        _vote: Vec<WeightedVoteOption>,
    ) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    /// If the caller has any delegations, withdraw all rewards from those delegations and
    /// send the tokens to the caller.
    /// NOTE: must make sure not to release unbonded tokens
    #[msg(exec)]
    fn withdraw_rewards(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        // TODO: track all validators
        let validators = vec!["todo".to_string()];

        // withdraw all delegations to the owner (set as withdrawl address in instantiate)
        let msgs = validators
            .into_iter()
            .map(|validator| DistributionMsg::WithdrawDelegatorReward { validator });
        let res = Response::new().add_messages(msgs);
        Ok(res)
    }

    /// unstakes the given amount from the given validator on behalf of the calling user.
    /// returns an error if the user doesn't have such stake.
    /// after unbonding period, it will allow the user to claim the tokens (returning to vault)
    #[msg(exec)]
    fn unstake(
        &self,
        ctx: ExecCtx,
        _validator: String,
        _amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    /// releases any tokens that have fully unbonded from a previous unstake.
    /// this will go back to the parent via `release_proxy_stake`
    /// error if the proxy doesn't have any liquid tokens
    #[msg(exec)]
    fn release_unbonded(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let cfg = self.config.load(ctx.deps.storage)?;
        ensure_eq!(cfg.owner, ctx.info.sender, ContractError::Unauthorized {});

        todo!()
    }

    #[msg(query)]
    fn config(&self, _ctx: QueryCtx) -> Result<ConfigResponse, ContractError> {
        todo!()
    }

    /// Returns all pending unbonding for this account
    /// TODO: can we do that with contract API?
    /// Or better they use cosmjs native delegation queries with this proxy address
    #[msg(query)]
    fn unbonding(&self, _ctx: QueryCtx, _account: String) -> Result<ClaimsResponse, ContractError> {
        todo!()
    }
}

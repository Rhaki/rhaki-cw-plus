use anyhow::Result as AnyResult;
use cw_multi_test::{AppResponse, BankSudo, CosmosRouter, SudoMsg};
use std::{collections::BTreeMap, vec};
use thiserror::Error;

use cosmwasm_std::{Addr, Api, BankMsg, BlockInfo, Coin, CosmosMsg, CustomQuery, Storage, Uint128};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenFactoryError {
    #[error("Invalid owner: {owner} != {sender} for {denom}")]
    InvalidOwner {
        owner: String,
        sender: String,
        denom: String,
    },

    #[error("Invalid denom format: {denom} is not in the format of 'factory/owner/subdenom'")]
    InvalidDenom { denom: String },

    #[error("Denom not existing: {denom} is not existing")]
    DenomNotExisting { denom: String },

    #[error("Denom alredy existing: {denom} alredy existing")]
    DenomAlredyExisting { denom: String },
}

pub type TokenFactoryResult<T> = Result<T, TokenFactoryError>;

pub struct CTokenFactoryFee {
    pub fee: Vec<Coin>,
    pub fee_collector: Addr,
}

#[derive(Default)]
pub struct CTokenFactory {
    pub fee_creation: Option<CTokenFactoryFee>,
    pub token_precisions: BTreeMap<String, u8>,
    pub supplies: BTreeMap<String, Uint128>,
}

impl CTokenFactory {
    fn assert_owner(&self, sender: &Addr, denom: &str) -> TokenFactoryResult<()> {
        let (owner, _) = self.try_parse(denom)?;

        if sender.to_string() != owner {
            return Err(TokenFactoryError::InvalidOwner {
                owner,
                sender: sender.to_string(),
                denom: denom.to_string(),
            });
        }

        Ok(())
    }

    fn try_parse(&self, denom: &str) -> TokenFactoryResult<(String, String)> {
        let parts: Vec<&str> = denom.split("/").collect();

        if parts.len() != 3 {
            return Err(TokenFactoryError::InvalidDenom {
                denom: denom.to_string(),
            });
        }

        return Ok((parts[1].to_string(), parts[2].to_string()));
    }

    fn build_denom(&self, sender: &Addr, subdenom: &str) -> String {
        format!("factory/{}/{}", sender, subdenom)
    }

    pub fn set_fee_creation(&mut self, fee: Vec<Coin>, fee_collector: Addr) {
        self.fee_creation = Some(CTokenFactoryFee { fee, fee_collector });
    }

    pub fn clear_fee_creation(&mut self) {
        self.fee_creation = None;
    }
}

// Execute

impl CTokenFactory {
    pub fn run_msg_mint<ExecC, QueryC: CustomQuery>(
        &mut self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        denom: String,
        amount: Uint128,
        to: String,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &denom)?;
        let mut supply =
            self.supplies
                .get(&denom)
                .cloned()
                .ok_or(TokenFactoryError::DenomNotExisting {
                    denom: denom.clone(),
                })?;

        supply += amount;

        self.supplies.insert(denom.clone(), supply);

        router.sudo(
            api,
            storage,
            block,
            SudoMsg::Bank(BankSudo::Mint {
                to_address: to.to_string(),
                amount: vec![Coin::new(amount.u128(), denom)],
            }),
        )
    }

    pub fn run_create_denom<ExecC, QueryC: CustomQuery>(
        &mut self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        sub_denom: String,
    ) -> AnyResult<AppResponse> {
        let denom = self.build_denom(&sender, &sub_denom);
        if self.supplies.get(&denom).is_some() {
            return Err(TokenFactoryError::DenomAlredyExisting {
                denom: denom.clone(),
            })?;
        }

        self.supplies.insert(denom.clone(), Uint128::zero());

        if let Some(fee_creation) = &self.fee_creation {
            router
                .execute(
                    api,
                    storage,
                    block,
                    sender,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: fee_creation.fee_collector.to_string(),
                        amount: fee_creation.fee.clone(),
                    }),
                )
                .map_err(|e| anyhow::anyhow!("Error on gather fee for denom creation: {}", e))
        } else {
            Ok(AppResponse::default())
        }
    }

    pub fn run_burn_denom<ExecC, QueryC: CustomQuery>(
        &mut self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &denom)?;
        let mut supply =
            self.supplies
                .get(&denom)
                .cloned()
                .ok_or(TokenFactoryError::DenomNotExisting {
                    denom: denom.clone(),
                })?;

        supply -= amount;
        self.supplies.insert(denom.clone(), supply);

        let burn_from_address = api.addr_validate(&burn_from_address)?;

        router.execute(
            api,
            storage,
            block,
            burn_from_address,
            CosmosMsg::Bank(BankMsg::Burn {
                amount: vec![Coin::new(amount.u128(), denom)],
            }),
        )
    }
}

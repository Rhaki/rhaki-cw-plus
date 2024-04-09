use anyhow::Result as AnyResult;
use cosmwasm_schema::cw_serde;
use cw_multi_test::{AppResponse, BankSudo, CosmosRouter, SudoMsg};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata;
use osmosis_std::types::{
    cosmos::bank::v1beta1::Metadata, osmosis::tokenfactory::v1beta1::MsgChangeAdmin,
};
use std::{collections::BTreeMap, vec};
use thiserror::Error;

use cosmwasm_std::{Addr, Api, BankMsg, BlockInfo, Coin, CosmosMsg, CustomQuery, Storage, Uint128};

use crate::traits::IntoAddr;

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

#[cw_serde]
pub struct CTokenFactoryFee {
    pub fee: Vec<Coin>,
    pub fee_collector: Addr,
}

#[derive(Default)]
#[cw_serde]
pub struct CTokenFactory {
    pub fee_creation: Option<CTokenFactoryFee>,
    pub token_precisions: BTreeMap<String, u8>,
    pub supplies: BTreeMap<String, Uint128>,
    pub metadata: BTreeMap<String, Metadata>,
    pub admin: BTreeMap<String, Addr>,
}

impl CTokenFactory {
    fn assert_owner(&self, sender: &Addr, denom: &str) -> TokenFactoryResult<()> {
        let owner = self
            .admin
            .get(denom)
            .ok_or(TokenFactoryError::DenomNotExisting {
                denom: denom.to_string(),
            })?;

        if owner != sender {
            Err(TokenFactoryError::InvalidOwner {
                owner: owner.to_string(),
                sender: sender.to_string(),
                denom: denom.to_string(),
            })
        } else {
            Ok(())
        }
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
    ) -> AnyResult<RunCreateDenomResponse> {
        let denom = self.build_denom(&sender, &sub_denom);
        if self.supplies.get(&denom).is_some() {
            return Err(TokenFactoryError::DenomAlredyExisting {
                denom: denom.clone(),
            })?;
        }

        self.supplies.insert(denom.clone(), Uint128::zero());

        self.admin.insert(denom.clone(), sender.clone());

        let response = if let Some(fee_creation) = &self.fee_creation {
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
                .map_err(|e| anyhow::anyhow!("Error on gather fee for denom creation: {}", e))?
        } else {
            AppResponse::default()
        };

        Ok(RunCreateDenomResponse { response, denom })
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

    pub fn run_set_denom_metadata<ExecC, QueryC: CustomQuery>(
        &mut self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        msg: MsgSetDenomMetadata,
    ) -> AnyResult<AppResponse> {
        if let Some(metadata) = msg.metadata {
            let denom = metadata.base.clone();

            self.assert_owner(&sender, &denom)?;

            self.metadata.insert(denom, metadata);
        }
        AnyResult::Ok(AppResponse::default())
    }

    pub fn run_change_admin<ExecC, QueryC: CustomQuery>(
        &mut self,
        api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        msg: MsgChangeAdmin,
    ) -> AnyResult<AppResponse> {
        self.assert_owner(&sender, &msg.denom)?;

        self.admin.insert(msg.denom, msg.new_admin.into_addr(api)?);

        Ok(AppResponse::default())
    }
}

pub struct RunCreateDenomResponse {
    pub response: AppResponse,
    pub denom: String,
}

// pub struct Metadata {}

use std::{collections::HashMap, rc::Rc};

use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::{
    analyzing::Analyzable,
    ast::{DataExpr, Scope},
    ir,
    lowering::IntoLower,
    parsing::{AstNode, Error, Rule},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoteDelegationCertificate {
    pub drep: DataExpr,
    pub stake: DataExpr,
}

impl AstNode for VoteDelegationCertificate {
    const RULE: Rule = Rule::cardano_vote_delegation_certificate;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        Ok(VoteDelegationCertificate {
            drep: DataExpr::parse(inner.next().unwrap())?,
            stake: DataExpr::parse(inner.next().unwrap())?,
        })
    }
}

impl Analyzable for VoteDelegationCertificate {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), crate::analyzing::Error> {
        self.drep.analyze(parent.clone())?;
        self.stake.analyze(parent.clone())?;

        Ok(())
    }
}

impl IntoLower for VoteDelegationCertificate {
    type Output = ir::AdHocDirective;

    fn into_lower(&self) -> Result<Self::Output, crate::lowering::Error> {
        Ok(ir::AdHocDirective {
            name: "vote_delegation_certificate".to_string(),
            data: HashMap::from([
                ("drep".to_string(), self.drep.into_lower()?),
                ("stake".to_string(), self.stake.into_lower()?),
            ]),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StakeDelegationCertificate {
    pub pool: DataExpr,
    pub stake: DataExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardanoBlock {
    VoteDelegationCertificate(VoteDelegationCertificate),
    StakeDelegationCertificate(StakeDelegationCertificate),
}

impl AstNode for CardanoBlock {
    const RULE: Rule = Rule::cardano_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        Ok(CardanoBlock::VoteDelegationCertificate(
            VoteDelegationCertificate::parse(inner.next().unwrap())?,
        ))
    }
}

impl Analyzable for CardanoBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), crate::analyzing::Error> {
        match self {
            CardanoBlock::VoteDelegationCertificate(x) => x.analyze(parent),
            _ => todo!(),
        }
    }
}

impl IntoLower for CardanoBlock {
    type Output = ir::AdHocDirective;

    fn into_lower(&self) -> Result<Self::Output, crate::lowering::Error> {
        match self {
            CardanoBlock::VoteDelegationCertificate(x) => x.into_lower(),
            _ => todo!(),
        }
    }
}

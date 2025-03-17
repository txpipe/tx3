use std::{collections::HashMap, rc::Rc};

use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::{
    analyzing::{Analyzable, AnalyzeReport},
    ast::{DataExpr, Scope, Span},
    ir,
    lowering::IntoLower,
    parsing::{AstNode, Error, Rule},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoteDelegationCertificate {
    pub drep: DataExpr,
    pub stake: DataExpr,
    pub span: Span,
}

impl AstNode for VoteDelegationCertificate {
    const RULE: Rule = Rule::cardano_vote_delegation_certificate;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let span = pair.as_span().into();
        let mut inner = pair.into_inner();

        Ok(VoteDelegationCertificate {
            drep: DataExpr::parse(inner.next().unwrap())?,
            stake: DataExpr::parse(inner.next().unwrap())?,
            span,
        })
    }

    fn span(&self) -> &Span {
        &self.span
    }
}

impl Analyzable for VoteDelegationCertificate {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> AnalyzeReport {
        let drep = self.drep.analyze(parent.clone());
        let stake = self.stake.analyze(parent.clone());

        drep + stake
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
    pub span: Span,
}

impl AstNode for StakeDelegationCertificate {
    const RULE: Rule = Rule::cardano_stake_delegation_certificate;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let span = pair.as_span().into();
        let mut inner = pair.into_inner();

        Ok(StakeDelegationCertificate {
            pool: DataExpr::parse(inner.next().unwrap())?,
            stake: DataExpr::parse(inner.next().unwrap())?,
            span,
        })
    }

    fn span(&self) -> &Span {
        &self.span
    }
}

impl Analyzable for StakeDelegationCertificate {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> AnalyzeReport {
        let pool = self.pool.analyze(parent.clone());
        let stake = self.stake.analyze(parent.clone());

        pool + stake
    }
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

    fn span(&self) -> &Span {
        match self {
            CardanoBlock::VoteDelegationCertificate(x) => x.span(),
            CardanoBlock::StakeDelegationCertificate(x) => x.span(),
        }
    }
}

impl Analyzable for CardanoBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> AnalyzeReport {
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

use std::rc::Rc;

use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::{
    analyze::{Scope, Symbol},
    parse::{Error as ParseError, Rule},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub value: String,

    // analysis
    #[serde(skip)]
    pub symbol: Option<Symbol>,
}

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            symbol: None,
        }
    }
}

impl AstNode for Identifier {
    const RULE: Rule = Rule::identifier;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(Identifier {
            value: pair.as_str().to_string(),
            symbol: None,
        })
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

pub trait AstNode: Sized {
    const RULE: Rule;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub txs: Vec<TxDef>,
    pub types: Vec<TypeDef>,
    pub assets: Vec<AssetDef>,
    pub parties: Vec<PartyDef>,
    pub policies: Vec<PolicyDef>,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

impl AstNode for Program {
    const RULE: Rule = Rule::program;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut program = Self {
            txs: Vec::new(),
            assets: Vec::new(),
            types: Vec::new(),
            parties: Vec::new(),
            policies: Vec::new(),
            scope: None,
        };

        for pair in inner {
            match pair.as_rule() {
                Rule::tx_def => program.txs.push(TxDef::parse(pair)?),
                Rule::asset_def => program.assets.push(AssetDef::parse(pair)?),
                Rule::type_def => program.types.push(TypeDef::parse(pair)?),
                Rule::party_def => program.parties.push(PartyDef::parse(pair)?),
                Rule::policy_def => program.policies.push(PolicyDef::parse(pair)?),
                Rule::EOI => break,
                x => unreachable!("Unexpected rule in program: {:?}", x),
            }
        }

        Ok(program)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterList {
    pub parameters: Vec<ParamDef>,
}

impl AstNode for ParameterList {
    const RULE: Rule = Rule::parameter_list;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut parameters = Vec::new();

        for param in inner {
            let mut inner = param.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let r#type = Type::parse(inner.next().unwrap())?;

            parameters.push(ParamDef { name, r#type });
        }

        Ok(ParameterList { parameters })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxDef {
    pub name: String,
    pub parameters: ParameterList,
    pub inputs: Vec<InputBlock>,
    pub outputs: Vec<OutputBlock>,
    pub burn: Option<BurnBlock>,
    pub mint: Option<MintBlock>,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

impl AstNode for TxDef {
    const RULE: Rule = Rule::tx_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = inner.next().unwrap().as_str().to_string();
        let parameters = ParameterList::parse(inner.next().unwrap())?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut burn = None;
        let mut mint = None;

        for item in inner {
            match item.as_rule() {
                Rule::input_block => inputs.push(InputBlock::parse(item)?),
                Rule::output_block => outputs.push(OutputBlock::parse(item)?),
                Rule::burn_block => burn = Some(BurnBlock::parse(item)?),
                Rule::mint_block => mint = Some(MintBlock::parse(item)?),
                x => unreachable!("Unexpected rule in tx_def: {:?}", x),
            }
        }

        Ok(TxDef {
            name,
            parameters,
            inputs,
            outputs,
            burn,
            mint,
            scope: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StringLiteral {
    pub value: String,
}

impl StringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl AstNode for StringLiteral {
    const RULE: Rule = Rule::string;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(StringLiteral {
            value: pair.as_str()[1..pair.as_str().len() - 1].to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HexStringLiteral {
    pub value: String,
}

impl HexStringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl AstNode for HexStringLiteral {
    const RULE: Rule = Rule::hex_string;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(HexStringLiteral {
            value: pair.as_str()[2..].to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputBlockField {
    From(DataExpr),
    DatumIs(Type),
    MinAmount(AssetExpr),
    Redeemer(DataExpr),
    Ref(DataExpr),
}

impl InputBlockField {
    fn key(&self) -> &str {
        match self {
            InputBlockField::From(_) => "from",
            InputBlockField::DatumIs(_) => "datum_is",
            InputBlockField::MinAmount(_) => "min_amount",
            InputBlockField::Redeemer(_) => "redeemer",
            InputBlockField::Ref(_) => "ref",
        }
    }
}

impl AstNode for InputBlockField {
    const RULE: Rule = Rule::input_block_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::input_block_from => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::From(DataExpr::parse(pair)?);
                Ok(x)
            }
            Rule::input_block_datum_is => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::DatumIs(Type::parse(pair)?);
                Ok(x)
            }
            Rule::input_block_min_amount => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::MinAmount(AssetExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::input_block_redeemer => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::Redeemer(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::input_block_ref => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::Ref(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            x => unreachable!("Unexpected rule in input_block: {:?}", x),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputBlock {
    pub name: String,
    pub is_many: bool,
    pub fields: Vec<InputBlockField>,
}

impl InputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&InputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

impl AstNode for InputBlock {
    const RULE: Rule = Rule::input_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(|x| InputBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(InputBlock {
            name,
            is_many: false,
            fields,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputBlockField {
    To(Box<DataExpr>),
    Amount(Box<AssetExpr>),
    Datum(Box<DataExpr>),
}

impl OutputBlockField {
    fn key(&self) -> &str {
        match self {
            OutputBlockField::To(_) => "to",
            OutputBlockField::Amount(_) => "amount",
            OutputBlockField::Datum(_) => "datum",
        }
    }
}

impl AstNode for OutputBlockField {
    const RULE: Rule = Rule::output_block_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::output_block_to => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::To(Box::new(DataExpr::parse(pair)?));
                Ok(x)
            }
            Rule::output_block_amount => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::Amount(AssetExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::output_block_datum => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::Datum(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            x => unreachable!("Unexpected rule in output_block_field: {:?}", x),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputBlock {
    pub name: Option<String>,
    pub fields: Vec<OutputBlockField>,
}

impl OutputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&OutputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

impl AstNode for OutputBlock {
    const RULE: Rule = Rule::output_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let has_name = inner
            .peek()
            .map(|x| x.as_rule() == Rule::identifier)
            .unwrap_or_default();

        let name = has_name.then(|| inner.next().unwrap().as_str().to_string());

        let fields = inner
            .map(|x| OutputBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OutputBlock { name, fields })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MintBlock {
    pub amount: Box<AssetExpr>,
}

impl AstNode for MintBlock {
    const RULE: Rule = Rule::mint_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut amount = None;

        for item in inner {
            match item.as_rule() {
                Rule::mint_block_amount => {
                    amount = Some(AssetExpr::parse(item.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in mint_block: {:?}", x),
            }
        }

        Ok(MintBlock {
            amount: amount.unwrap(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BurnBlock {
    pub amount: Box<AssetExpr>,
}

impl AstNode for BurnBlock {
    const RULE: Rule = Rule::burn_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut amount = None;

        for item in inner {
            match item.as_rule() {
                Rule::burn_block_amount => {
                    amount = Some(AssetExpr::parse(item.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in burn_block: {:?}", x),
            }
        }

        Ok(BurnBlock {
            amount: amount.unwrap(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RecordField {
    pub name: String,
    pub typ: Type,
}

impl RecordField {
    pub fn new(name: &str, typ: Type) -> Self {
        Self {
            name: name.to_string(),
            typ,
        }
    }
}

impl AstNode for RecordField {
    const RULE: Rule = Rule::record_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        let typ = Type::parse(inner.next().unwrap())?;

        Ok(RecordField {
            name: identifier,
            typ,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PartyDef {
    pub name: String,
}

impl AstNode for PartyDef {
    const RULE: Rule = Rule::party_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(PartyDef { name: identifier })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyField {
    pub name: String,
    pub party_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDef {
    pub name: String,
    pub value: PolicyValue,
}

impl AstNode for PolicyDef {
    const RULE: Rule = Rule::policy_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let value = PolicyValue::parse(inner.next().unwrap())?;

        Ok(PolicyDef { name, value })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyValue {
    Import(StringLiteral),
    HexString(HexStringLiteral),
}

impl AstNode for PolicyValue {
    const RULE: Rule = Rule::policy_value;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::policy_import => Ok(PolicyValue::Import(StringLiteral::parse(
                pair.into_inner().next().unwrap(),
            )?)),
            Rule::hex_string => Ok(PolicyValue::HexString(HexStringLiteral::parse(pair)?)),
            x => unreachable!("Unexpected rule in policy_value: {:?}", x),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetConstructor {
    pub r#type: Identifier,
    pub amount: Box<DataExpr>,
    pub asset_name: Option<Box<DataExpr>>,
}

impl AstNode for AssetConstructor {
    const RULE: Rule = Rule::asset_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let amount = DataExpr::parse(inner.next().unwrap())?;
        let name = inner.next().map(|x| DataExpr::parse(x)).transpose()?;

        Ok(AssetConstructor {
            r#type,
            amount: Box::new(amount),
            asset_name: name.map(|x| Box::new(x)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetBinaryOp {
    pub left: Box<AssetExpr>,
    pub operator: BinaryOperator,
    pub right: Box<AssetExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetExpr {
    Constructor(AssetConstructor),
    BinaryOp(AssetBinaryOp),
    PropertyAccess(PropertyAccess),
    Identifier(Identifier),
}

impl AssetExpr {
    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::Identifier(Identifier::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::Constructor(AssetConstructor::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::asset_constructor => AssetExpr::constructor_parse(pair),
            Rule::property_access => AssetExpr::property_access_parse(pair),
            Rule::identifier => AssetExpr::identifier_parse(pair),
            x => Err(ParseError::UnexpectedRule(x)),
        }
    }
}

impl AstNode for AssetExpr {
    const RULE: Rule = Rule::asset_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::term_parse(inner.next().unwrap())?;

        while let Some(term) = inner.next() {
            let operator = BinaryOperator::parse(term)?;
            let next_expr = Self::term_parse(inner.next().unwrap())?;

            final_expr = AssetExpr::BinaryOp(AssetBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyAccess {
    pub object: Identifier,
    pub path: Vec<Identifier>,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

impl PropertyAccess {
    pub fn new(object: &str, path: &[&str]) -> Self {
        Self {
            object: Identifier::new(object),
            path: path.iter().map(|x| Identifier::new(*x)).collect(),
            scope: None,
        }
    }
}

impl PropertyAccess {
    /// Shift the property access to the next property in the path.
    pub fn shift(mut self) -> Option<Self> {
        if self.path.is_empty() {
            return None;
        }

        let new_object = self.path.remove(0);
        self.object = new_object;

        Some(self)
    }
}

impl AstNode for PropertyAccess {
    const RULE: Rule = Rule::property_access;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let object = Identifier::parse(inner.next().unwrap())?;

        let mut identifiers = Vec::new();
        identifiers.push(Identifier::parse(inner.next().unwrap())?);

        for identifier in inner {
            identifiers.push(Identifier::parse(identifier)?);
        }

        Ok(PropertyAccess {
            object,
            path: identifiers,
            scope: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordConstructorField {
    pub name: Identifier,
    pub value: Box<DataExpr>,
}

impl AstNode for RecordConstructorField {
    const RULE: Rule = Rule::record_constructor_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;
        let value = DataExpr::parse(inner.next().unwrap())?;

        Ok(RecordConstructorField {
            name,
            value: Box::new(value),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumConstructor {
    pub r#type: Identifier,
    pub variant: Option<Identifier>,
    pub fields: Vec<RecordConstructorField>,
    pub spread: Option<Box<DataExpr>>,
}

impl DatumConstructor {
    fn record_constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;

        let mut fields = Vec::new();
        let mut spread = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::record_constructor_field => {
                    fields.push(RecordConstructorField::parse(pair)?);
                }
                Rule::spread_expression => {
                    spread = Some(DataExpr::parse(pair.into_inner().next().unwrap())?);
                }
                x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
            }
        }

        Ok(DatumConstructor {
            r#type,
            variant: None,
            fields,
            spread: spread.map(|x| Box::new(x)),
        })
    }

    fn variant_constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let variant = Identifier::parse(inner.next().unwrap())?;

        let mut fields = Vec::new();
        let mut spread = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::record_constructor_field => {
                    fields.push(RecordConstructorField::parse(pair)?);
                }
                Rule::spread_expression => {
                    spread = Some(DataExpr::parse(pair.into_inner().next().unwrap())?);
                }
                x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
            }
        }

        Ok(DatumConstructor {
            r#type,
            variant: Some(variant),
            fields,
            spread: spread.map(|x| Box::new(x)),
        })
    }

    fn unit_constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DatumConstructor {
            r#type: Identifier::new("Unit".to_string()),
            variant: None,
            fields: vec![],
            spread: None,
        })
    }
}

impl AstNode for DatumConstructor {
    const RULE: Rule = Rule::datum_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::variant_constructor => DatumConstructor::variant_constructor_parse(pair),
            Rule::record_constructor => DatumConstructor::record_constructor_parse(pair),
            Rule::unit_constructor => DatumConstructor::unit_constructor_parse(pair),
            x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataBinaryOp {
    pub left: Box<DataExpr>,
    pub operator: BinaryOperator,
    pub right: Box<DataExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataExpr {
    None,
    Number(i64),
    Bool(bool),
    String(StringLiteral),
    HexString(HexStringLiteral),
    Constructor(DatumConstructor),
    Identifier(Identifier),
    PropertyAccess(PropertyAccess),
    BinaryOp(DataBinaryOp),
}

impl DataExpr {
    fn number_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Number(pair.as_str().parse().unwrap()))
    }

    fn bool_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Bool(pair.as_str().parse().unwrap()))
    }

    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Identifier(Identifier::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Constructor(DatumConstructor::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::number => DataExpr::number_parse(pair),
            Rule::string => Ok(DataExpr::String(StringLiteral::parse(pair)?)),
            Rule::bool => DataExpr::bool_parse(pair),
            Rule::hex_string => Ok(DataExpr::HexString(HexStringLiteral::parse(pair)?)),
            Rule::record_constructor => DataExpr::constructor_parse(pair),
            Rule::variant_constructor => DataExpr::constructor_parse(pair),
            Rule::unit_constructor => DataExpr::constructor_parse(pair),
            Rule::identifier => DataExpr::identifier_parse(pair),
            Rule::property_access => DataExpr::property_access_parse(pair),
            x => Err(ParseError::UnexpectedRule(x)),
        }
    }
}

impl AstNode for DataExpr {
    const RULE: Rule = Rule::data_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::term_parse(inner.next().unwrap())?;

        while let Some(term) = inner.next() {
            let operator = BinaryOperator::parse(term)?;
            let next_expr = Self::term_parse(inner.next().unwrap())?;

            final_expr = DataExpr::BinaryOp(DataBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

impl AstNode for BinaryOperator {
    const RULE: Rule = Rule::binary_operator;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_str() {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            x => Err(ParseError::InvalidBinaryOperator(x.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Bytes,
    Custom(Identifier),
}

impl AstNode for Type {
    const RULE: Rule = Rule::r#type;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "Bytes" => Ok(Type::Bytes),
            t => Ok(Type::Custom(Identifier::new(t.to_string()))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParamDef {
    pub name: String,
    pub r#type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantDef {
    pub name: String,
    pub cases: Vec<VariantCase>,
}

impl AstNode for VariantDef {
    const RULE: Rule = Rule::variant_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let cases = inner
            .map(VariantCase::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(VariantDef {
            name: identifier,
            cases,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantCase {
    pub name: String,
    pub fields: Vec<RecordField>,
}

impl VariantCase {
    fn struct_case_parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(RecordField::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: identifier,
            fields,
        })
    }

    fn unit_case_parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(Self {
            name: identifier,
            fields: vec![],
        })
    }
}

impl AstNode for VariantCase {
    const RULE: Rule = Rule::variant_case;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let case = match pair.as_rule() {
            Rule::variant_case_struct => Self::struct_case_parse(pair),
            Rule::variant_case_tuple => todo!("parse variant case tuple"),
            Rule::variant_case_unit => Self::unit_case_parse(pair),
            x => unreachable!("Unexpected rule in datum_variant: {:?}", x),
        }?;

        Ok(case)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<RecordField>,
}

impl AstNode for RecordDef {
    const RULE: Rule = Rule::record_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(RecordField::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: identifier,
            fields,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeDef {
    Variant(VariantDef),
    Record(RecordDef),
    //Alias(AliasDef),
}

impl AstNode for TypeDef {
    const RULE: Rule = Rule::type_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::variant_def => Ok(TypeDef::Variant(VariantDef::parse(inner)?)),
            Rule::record_def => Ok(TypeDef::Record(RecordDef::parse(inner)?)),
            x => unreachable!("Unexpected rule in type_def: {:?}", x),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetDef {
    pub name: String,
    pub policy: String,
    pub asset_name: Option<String>,
}

impl AstNode for AssetDef {
    const RULE: Rule = Rule::asset_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();
        let policy = inner.next().unwrap().as_str().to_string();
        let asset_name = inner.next().map(|x| x.as_str().to_string());

        Ok(AssetDef {
            name: identifier,
            policy,
            asset_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pest::Parser;

    macro_rules! input_to_ast_check {
        ($ast:ty, $name:expr, $input:expr, $expected:expr) => {
            paste::paste! {
                #[test]
                fn [<test_parse_ $ast:snake _ $name>]() {
                    let pairs = crate::parse::Tx3Grammar::parse(<$ast>::RULE, $input).unwrap();
                    let single_match = pairs.into_iter().next().unwrap();
                    let result = <$ast>::parse(single_match).unwrap();

                    assert_eq!(result, $expected);
                }
            }
        };
    }

    input_to_ast_check!(BinaryOperator, "plus", "+", BinaryOperator::Add);

    input_to_ast_check!(BinaryOperator, "minus", "-", BinaryOperator::Subtract);

    input_to_ast_check!(
        RecordDef,
        "record_def",
        "record MyRecord {
            field1: Int,
            field2: Bytes,
        }",
        RecordDef {
            name: "MyRecord".to_string(),
            fields: vec![
                RecordField::new("field1", Type::Int),
                RecordField::new("field2", Type::Bytes)
            ],
        }
    );

    input_to_ast_check!(
        VariantDef,
        "variant_def",
        "variant MyVariant {
            Case1 {
                field1: Int,
                field2: Bytes,
            },
            Case2,
        }",
        VariantDef {
            name: "MyVariant".to_string(),
            cases: vec![
                VariantCase {
                    name: "Case1".to_string(),
                    fields: vec![
                        RecordField::new("field1", Type::Int),
                        RecordField::new("field2", Type::Bytes)
                    ],
                },
                VariantCase {
                    name: "Case2".to_string(),
                    fields: vec![],
                },
            ],
        }
    );

    input_to_ast_check!(
        TypeDef,
        "type_def",
        "variant MyType {
            Case1 {
                field1: Int,
                field2: Bytes,
            },
            Case2,
        }",
        TypeDef::Variant(VariantDef {
            name: "MyType".to_string(),
            cases: vec![
                VariantCase {
                    name: "Case1".to_string(),
                    fields: vec![
                        RecordField::new("field1", Type::Int),
                        RecordField::new("field2", Type::Bytes)
                    ],
                },
                VariantCase {
                    name: "Case2".to_string(),
                    fields: vec![],
                },
            ],
        })
    );

    input_to_ast_check!(
        StringLiteral,
        "literal_string",
        "\"Hello, world!\"",
        StringLiteral::new("Hello, world!".to_string())
    );

    input_to_ast_check!(
        HexStringLiteral,
        "hex_string",
        "0xAFAFAF",
        HexStringLiteral::new("AFAFAF".to_string())
    );

    input_to_ast_check!(
        StringLiteral,
        "literal_string_address",
        "\"addr1qx234567890abcdefghijklmnopqrstuvwxyz\"",
        StringLiteral::new("addr1qx234567890abcdefghijklmnopqrstuvwxyz".to_string())
    );

    input_to_ast_check!(
        DataExpr,
        "identifier",
        "my_party",
        DataExpr::Identifier(Identifier::new("my_party".to_string()))
    );

    input_to_ast_check!(DataExpr, "literal_bool_true", "true", DataExpr::Bool(true));

    input_to_ast_check!(
        DataExpr,
        "literal_bool_false",
        "false",
        DataExpr::Bool(false)
    );

    input_to_ast_check!(
        PropertyAccess,
        "single_property",
        "subject.property",
        PropertyAccess::new("subject", &["property"])
    );

    input_to_ast_check!(
        PropertyAccess,
        "multiple_properties",
        "subject.property.subproperty",
        PropertyAccess::new("subject", &["property", "subproperty"])
    );

    input_to_ast_check!(
        PolicyDef,
        "policy_def_hex",
        "policy MyPolicy = 0xAFAFAF;",
        PolicyDef {
            name: "MyPolicy".to_string(),
            value: PolicyValue::HexString(HexStringLiteral::new("AFAFAF".to_string())),
        }
    );

    input_to_ast_check!(
        PolicyDef,
        "policy_def_import",
        "policy MyPolicy = import(\"aiken_code\");",
        PolicyDef {
            name: "MyPolicy".to_string(),
            value: PolicyValue::Import(StringLiteral::new("aiken_code".to_string())),
        }
    );

    input_to_ast_check!(
        AssetConstructor,
        "type_and_literal",
        "MyToken(15)",
        AssetConstructor {
            r#type: Identifier::new("MyToken"),
            amount: Box::new(DataExpr::Number(15)),
            asset_name: None,
        }
    );

    input_to_ast_check!(
        AssetConstructor,
        "type_and_literal_with_name",
        "MyClass(15, \"TokenName\")",
        AssetConstructor {
            r#type: Identifier::new("MyClass"),
            amount: Box::new(DataExpr::Number(15)),
            asset_name: Some(Box::new(DataExpr::String(StringLiteral::new(
                "TokenName".to_string(),
            )))),
        }
    );

    input_to_ast_check!(
        DataExpr,
        "addition",
        "5 + var1",
        DataExpr::BinaryOp(DataBinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(DataExpr::Number(5)),
            right: Box::new(DataExpr::Identifier(Identifier::new("var1"))),
        })
    );

    input_to_ast_check!(
        DatumConstructor,
        "unit_constructor",
        "()",
        DatumConstructor {
            r#type: Identifier::new("Unit"),
            variant: None,
            fields: vec![],
            spread: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "struct_record",
        "MyRecord {
            field1: 10,
            field2: abc,
        }",
        DatumConstructor {
            r#type: Identifier::new("MyRecord"),
            variant: None,
            fields: vec![
                RecordConstructorField {
                    name: Identifier::new("field1"),
                    value: Box::new(DataExpr::Number(10)),
                },
                RecordConstructorField {
                    name: Identifier::new("field2"),
                    value: Box::new(DataExpr::Identifier(Identifier::new("abc"))),
                },
            ],
            spread: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "struct_variant",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
        }",
        DatumConstructor {
            r#type: Identifier::new("ShipCommand"),
            variant: Some(Identifier::new("MoveShip")),
            fields: vec![
                RecordConstructorField {
                    name: Identifier::new("delta_x"),
                    value: Box::new(DataExpr::Identifier(Identifier::new("delta_x"))),
                },
                RecordConstructorField {
                    name: Identifier::new("delta_y"),
                    value: Box::new(DataExpr::Identifier(Identifier::new("delta_y"))),
                },
            ],
            spread: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "struct_variant_with_spread",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
            ...abc
        }",
        DatumConstructor {
            r#type: Identifier::new("ShipCommand"),
            variant: Some(Identifier::new("MoveShip")),
            fields: vec![
                RecordConstructorField {
                    name: Identifier::new("delta_x"),
                    value: Box::new(DataExpr::Identifier(Identifier::new("delta_x"))),
                },
                RecordConstructorField {
                    name: Identifier::new("delta_y"),
                    value: Box::new(DataExpr::Identifier(Identifier::new("delta_y"))),
                },
            ],
            spread: Some(Box::new(DataExpr::Identifier(Identifier::new(
                "abc".to_string()
            )))),
        }
    );

    input_to_ast_check!(
        OutputBlock,
        "output_block_anonymous",
        r#"output {
            to: my_party,
            amount: Ada(100),
        }"#,
        OutputBlock {
            name: None,
            fields: vec![
                OutputBlockField::To(Box::new(DataExpr::Identifier(Identifier::new(
                    "my_party".to_string(),
                )))),
                OutputBlockField::Amount(Box::new(AssetExpr::Constructor(AssetConstructor {
                    r#type: Identifier::new("Ada"),
                    amount: Box::new(DataExpr::Number(100)),
                    asset_name: None,
                }))),
            ],
        }
    );
}

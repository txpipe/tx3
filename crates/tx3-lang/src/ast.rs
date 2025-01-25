#[derive(Debug, Clone)]
pub struct TemplateData {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Vec<TxComponent>,
}

#[derive(Debug, Clone)]
pub enum TxComponent {
    Input(InputData),
    Output(OutputData),
}

#[derive(Debug, Clone)]
pub struct InputData {
    pub name: String,
    pub is_many: bool,
    pub from: Option<String>,
    pub datum_is: Option<String>,
    pub min_amount: Option<Box<Expr>>,
    pub redeemer: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct OutputData {
    pub to: String,
    pub amount: Option<Box<Expr>>,
    pub datum: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct DatumDef {
    pub name: String,
    pub fields: Vec<DatumField>,
}

#[derive(Debug, Clone)]
pub struct DatumField {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct PartyDef {
    pub name: String,
    pub fields: Vec<PartyField>,
}

#[derive(Debug, Clone)]
pub struct PartyField {
    pub name: String,
    pub party_type: String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Number(i64),
    String(String),
    None,

    // Template definition
    Template(TemplateData),

    // Datum definition
    Datum(DatumDef),

    // Party definition
    Party(PartyDef),

    // Field access (for supporting sub_field rule)
    SubField {
        base: Box<Expr>,
        fields: Vec<String>,
    },

    // Object/Datum construction
    DatumConstructor {
        name: String,
        fields: Vec<(String, Box<Expr>)>,
        spread: Option<Box<Expr>>,
    },

    // Binary operations
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },

    // Property access
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },

    // Variables and function calls
    Identifier(String),
    Call(CallData),
}

#[derive(Debug, Clone)]
pub struct CallData {
    pub function: Box<Expr>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Token,
    Datum,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
}

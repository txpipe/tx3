pub use pallas::codec::utils::Int;
pub use pallas::ledger::primitives::{BigInt, BoundedBytes, PlutusData};

pub trait IntoData {
    fn into_data(&self) -> PlutusData;
}

#[macro_export]
macro_rules! constr {
    ($tag:expr, $($field:expr),*) => {
        {
            use pallas::ledger::primitives::{Constr, PlutusData, MaybeIndefArray};

            let inner = Constr {
                tag: 121 + $tag,
                any_constructor: None,
                fields: MaybeIndefArray::Def(vec![$($field.into_data()),*]),
            };

            PlutusData::Constr(inner)
        }
    };
}

impl IntoData for () {
    fn into_data(&self) -> PlutusData {
        constr!(0,)
    }
}

impl IntoData for bool {
    fn into_data(&self) -> PlutusData {
        PlutusData::BoundedBytes(BoundedBytes::from(vec![*self as u8]))
    }
}

impl<const N: usize> IntoData for [u8; N] {
    fn into_data(&self) -> PlutusData {
        PlutusData::BoundedBytes(BoundedBytes::from(self.to_vec()))
    }
}

impl IntoData for Vec<u8> {
    fn into_data(&self) -> PlutusData {
        PlutusData::BoundedBytes(BoundedBytes::from(self.clone()))
    }
}

impl IntoData for u64 {
    fn into_data(&self) -> PlutusData {
        PlutusData::BigInt(BigInt::Int(Int::from(*self as i64)))
    }
}

impl IntoData for i64 {
    fn into_data(&self) -> PlutusData {
        PlutusData::BigInt(BigInt::Int(Int::from(*self)))
    }
}

mod test {
    use super::*;

    fn construct_constr() {
        let x = constr!(0, b"abc", vec![1, 2, 3]);
    }
}

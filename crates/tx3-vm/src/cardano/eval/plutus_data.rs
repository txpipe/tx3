pub use pallas::codec::utils::Int;
pub use pallas::ledger::primitives::{BigInt, BoundedBytes, Constr, MaybeIndefArray, PlutusData};

pub trait IntoData {
    fn into_data(&self) -> PlutusData;
}

#[macro_export]
macro_rules! constr {
    ($tag:expr, $($field:expr),*) => {
        {
            let fields = vec![$($field.into_data()),*];
            $crate::cardano::eval::plutus_data::constr($tag, fields)
        }
    };
}

pub fn constr(tag: u64, fields: Vec<PlutusData>) -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121 + tag,
        any_constructor: None,
        fields: MaybeIndefArray::Def(fields),
    })
}

impl IntoData for () {
    fn into_data(&self) -> PlutusData {
        constr!(0,)
    }
}

impl IntoData for PlutusData {
    fn into_data(&self) -> PlutusData {
        self.clone()
    }
}

impl IntoData for bool {
    fn into_data(&self) -> PlutusData {
        PlutusData::BoundedBytes(BoundedBytes::from(vec![*self as u8]))
    }
}

impl IntoData for &str {
    fn into_data(&self) -> PlutusData {
        PlutusData::BoundedBytes(BoundedBytes::from(self.as_bytes().to_vec()))
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

impl IntoData for i128 {
    fn into_data(&self) -> PlutusData {
        let int = Int::try_from(*self).unwrap();
        PlutusData::BigInt(BigInt::Int(int))
    }
}

mod test {
    use super::*;

    fn construct_constr() {
        let x = constr!(0, b"abc", vec![1, 2, 3]);
    }
}

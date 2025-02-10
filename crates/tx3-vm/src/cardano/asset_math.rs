use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{
    conway::{self, Value},
    AssetName, NonEmptyKeyValuePairs, NonZeroInt, PolicyId, PositiveCoin,
};
use std::collections::{hash_map::Entry, HashMap};

use crate::Error;

fn fold_assets<T>(
    acc: &mut HashMap<pallas::codec::utils::Bytes, T>,
    item: NonEmptyKeyValuePairs<pallas::codec::utils::Bytes, T>,
) where
    T: SafeAdd + Copy,
{
    for (key, value) in item.to_vec() {
        match acc.entry(key) {
            Entry::Occupied(mut entry) => {
                if let Some(new_val) = value.try_add(*entry.get()) {
                    entry.insert(new_val);
                } else {
                    entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }
}

pub fn fold_multiassets<T>(
    acc: &mut HashMap<Hash<28>, HashMap<pallas::codec::utils::Bytes, T>>,
    item: NonEmptyKeyValuePairs<Hash<28>, NonEmptyKeyValuePairs<pallas::codec::utils::Bytes, T>>,
) where
    T: SafeAdd + Copy,
{
    for (key, value) in item.to_vec() {
        let mut map = acc.remove(&key).unwrap_or_default();
        fold_assets(&mut map, value);
        acc.insert(key, map);
    }
}

pub fn aggregate_assets<T>(
    items: impl IntoIterator<Item = conway::Multiasset<T>>,
) -> Option<conway::Multiasset<T>>
where
    T: SafeAdd + Copy,
{
    let mut total_assets = HashMap::new();

    for assets in items {
        fold_multiassets(&mut total_assets, assets);
    }

    let total_assets_vec = total_assets
        .into_iter()
        .filter_map(|(key, assets)| {
            let assets_vec = assets.into_iter().collect();
            Some((key, NonEmptyKeyValuePairs::from_vec(assets_vec)?))
        })
        .collect();

    NonEmptyKeyValuePairs::from_vec(total_assets_vec)
}

pub fn aggregate_values(items: impl IntoIterator<Item = Value>) -> Value {
    let mut total_coin = 0;
    let mut assets = vec![];

    for value in items {
        match value {
            Value::Coin(x) => {
                total_coin += x;
            }
            Value::Multiasset(x, y) => {
                total_coin += x;
                assets.push(y);
            }
        }
    }

    if let Some(total_assets) = aggregate_assets(assets) {
        Value::Multiasset(total_coin, total_assets)
    } else {
        Value::Coin(total_coin)
    }
}

pub fn add_values(lhs: &Value, rhs: &Value) -> Result<Value, Error> {
    Ok(aggregate_values(vec![lhs.clone(), rhs.clone()]))
}

pub fn subtract_value(lhs: &Value, rhs: &Value) -> Result<Value, Error> {
    let (lhs_coin, lhs_assets) = match lhs {
        Value::Coin(c) => (*c, vec![]),
        Value::Multiasset(c, a) => (*c, a.iter().collect()),
    };

    let (rhs_coin, mut rhs_assets) = match rhs {
        Value::Coin(c) => (*c, HashMap::new()),
        Value::Multiasset(c, a) => {
            let flattened: HashMap<(&PolicyId, &AssetName), u64> = a
                .iter()
                .flat_map(|(policy, assets)| {
                    assets
                        .iter()
                        .map(move |(name, value)| ((policy, name), value.into()))
                })
                .collect();
            (*c, flattened)
        }
    };

    let Some(final_coin) = lhs_coin.checked_sub(rhs_coin) else {
        return Err(Error::OutputsTooHigh);
    };

    let mut final_assets = vec![];
    for (policy, assets) in lhs_assets {
        let mut policy_assets = vec![];
        for (name, value) in assets.iter() {
            let lhs_value: u64 = value.into();
            let rhs_value: u64 = rhs_assets.remove(&(policy, name)).unwrap_or_default();
            let Some(final_value) = lhs_value.checked_sub(rhs_value) else {
                return Err(Error::OutputsTooHigh);
            };
            if let Ok(final_coin) = final_value.try_into() {
                policy_assets.push((name.clone(), final_coin));
            }
        }
        if let Some(assets) = NonEmptyKeyValuePairs::from_vec(policy_assets) {
            final_assets.push((*policy, assets));
        }
    }

    if !rhs_assets.is_empty() {
        // We have an output which didn't come from any inputs
        return Err(Error::OutputsTooHigh);
    }

    if let Some(assets) = NonEmptyKeyValuePairs::from_vec(final_assets) {
        Ok(Value::Multiasset(final_coin, assets))
    } else {
        Ok(Value::Coin(final_coin))
    }
}

pub fn value_coin(value: &Value) -> u64 {
    match value {
        Value::Coin(x) => *x,
        Value::Multiasset(x, _) => *x,
    }
}

fn try_to_mint<F>(assets: conway::Multiasset<PositiveCoin>, f: F) -> Result<conway::Mint, Error>
where
    F: Fn(i64) -> Result<NonZeroInt, i64>,
{
    let mut new_assets = vec![];
    for (policy, asset) in assets {
        let mut new_asset = vec![];
        for (name, quantity) in asset {
            let quantity: u64 = quantity.into();
            if quantity > i64::MAX as u64 {
                return Err(Error::AssetValueTooHigh);
            }
            let quantity: NonZeroInt = f(quantity as i64).unwrap();
            new_asset.push((name, quantity));
        }
        let asset = NonEmptyKeyValuePairs::from_vec(new_asset).unwrap();
        new_assets.push((policy, asset));
    }

    Ok(NonEmptyKeyValuePairs::from_vec(new_assets).unwrap())
}

pub fn multiasset_coin_to_mint(
    assets: conway::Multiasset<PositiveCoin>,
) -> Result<conway::Mint, Error> {
    try_to_mint(assets, |quantity| quantity.try_into())
}

pub fn multiasset_coin_to_burn(
    assets: conway::Multiasset<PositiveCoin>,
) -> Result<conway::Mint, Error> {
    try_to_mint(assets, |quantity| (-quantity).try_into())
}

pub fn value_saturating_add_coin(value: Value, coin: i64) -> Value {
    match value {
        Value::Coin(x) => Value::Coin(x.saturating_add_signed(coin)),
        Value::Multiasset(x, assets) => Value::Multiasset(x.saturating_add_signed(coin), assets),
    }
}

pub trait SafeAdd: Sized {
    fn try_add(self, other: Self) -> Option<Self>;
}

impl SafeAdd for NonZeroInt {
    fn try_add(self, other: Self) -> Option<Self> {
        let lhs: i64 = self.into();
        let rhs: i64 = other.into();
        NonZeroInt::try_from(lhs.checked_add(rhs)?).ok()
    }
}

impl SafeAdd for PositiveCoin {
    fn try_add(self, other: Self) -> Option<Self> {
        let lhs: u64 = self.into();
        let rhs: u64 = other.into();
        PositiveCoin::try_from(lhs.checked_add(rhs)?).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;
    use pallas::ledger::primitives::conway::Value;

    #[test]
    fn test_add_values_coin_only() {
        let value_a = Value::Coin(100);
        let value_b = Value::Coin(200);

        let result = aggregate_values(vec![value_a, value_b]);

        assert_eq!(result, Value::Coin(300));
    }

    #[test]
    fn test_add_values_same_asset() {
        let policy_id =
            Hash::<28>::from_str("bb4bc871e84078de932d392186dd3093b8de93505178d88d89b7ac98")
                .unwrap();

        let asset_name = "pepe".as_bytes().to_vec();

        let value_a = Value::Multiasset(
            100,
            NonEmptyKeyValuePairs::Def(vec![(
                policy_id,
                NonEmptyKeyValuePairs::Def(vec![(
                    asset_name.clone().into(),
                    50.try_into().unwrap(),
                )]),
            )]),
        );
        let value_b = Value::Multiasset(
            200,
            NonEmptyKeyValuePairs::Def(vec![(
                policy_id,
                NonEmptyKeyValuePairs::Def(vec![(
                    asset_name.clone().into(),
                    30.try_into().unwrap(),
                )]),
            )]),
        );

        let result = aggregate_values(vec![value_a, value_b]);

        assert_eq!(
            result,
            Value::Multiasset(
                300,
                NonEmptyKeyValuePairs::Def(vec![(
                    policy_id,
                    NonEmptyKeyValuePairs::Def(vec![(
                        asset_name.clone().into(),
                        80.try_into().unwrap()
                    )]),
                )]),
            )
        );
    }
}

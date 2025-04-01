use pallas::ledger::primitives::{
    conway::{self, Value},
    Hash, NonZeroInt, PositiveCoin,
};
use std::collections::{btree_map::Entry, BTreeMap};

fn fold_assets<T>(
    acc: &mut BTreeMap<pallas::codec::utils::Bytes, T>,
    item: BTreeMap<pallas::codec::utils::Bytes, T>,
) where
    T: SafeAdd + Copy,
{
    for (key, value) in item.into_iter() {
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
    acc: &mut BTreeMap<Hash<28>, BTreeMap<pallas::codec::utils::Bytes, T>>,
    item: BTreeMap<Hash<28>, BTreeMap<pallas::codec::utils::Bytes, T>>,
) where
    T: SafeAdd + Copy,
{
    for (key, value) in item.into_iter() {
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
    let mut total_assets = BTreeMap::new();

    for assets in items {
        fold_multiassets(&mut total_assets, assets);
    }

    if total_assets.is_empty() {
        None
    } else {
        Some(total_assets)
    }
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
    use std::collections::BTreeMap;
    use std::str::FromStr as _;

    use super::*;
    use pallas::ledger::primitives::conway::Value;

    macro_rules! asset {
        ($policy:expr, $asset:expr, $amount:expr) => {{
            let mut aux = BTreeMap::new();
            aux.insert($asset, $amount);
            let mut asset = BTreeMap::new();
            asset.insert($policy, aux);
            asset
        }};
    }

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
            asset!(policy_id, asset_name.clone().into(), 50.try_into().unwrap()),
        );
        let value_b = Value::Multiasset(
            200,
            asset!(policy_id, asset_name.clone().into(), 30.try_into().unwrap()),
        );

        let result = aggregate_values(vec![value_a, value_b]);

        assert_eq!(
            result,
            Value::Multiasset(
                300,
                asset!(policy_id, asset_name.clone().into(), 80.try_into().unwrap()),
            )
        );
    }
}

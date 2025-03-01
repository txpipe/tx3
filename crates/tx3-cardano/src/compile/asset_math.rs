use pallas::ledger::primitives::{
    conway::{self, Value},
    Hash, NonEmptyKeyValuePairs, NonZeroInt, PositiveCoin,
};
use std::collections::{hash_map::Entry, HashMap};

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

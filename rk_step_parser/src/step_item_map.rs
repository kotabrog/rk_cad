use std::collections::HashMap;

use super::step_entity::{EntityId, SimpleEntity, StepEntity};
use super::step_item::{ConversionStepItemError, StepItem};

#[derive(Debug)]
pub struct StepItems {
    pub items: Vec<StepItem>,
}

/// `#id → Vec<StepItem>`  (still un‑linked, complex entities may
/// contribute multiple StepItems to the same id)
pub type StepItemMap = HashMap<EntityId, StepItems>;

#[derive(thiserror::Error, Debug)]
pub enum StepItemMapError {
    #[error("duplicate entity id #{0}")]
    DuplicateId(usize),

    #[error("failed to convert #{id} part: {source}")]
    ConvertPart {
        id: usize,
        #[source]
        source: ConversionStepItemError,
    },
}

impl StepItems {
    pub fn get_single(&self) -> Option<&StepItem> {
        if self.items.len() == 1 {
            Some(&self.items[0])
        } else {
            None
        }
    }

    pub fn get_multiple(&self) -> Option<&Vec<StepItem>> {
        if self.items.len() > 1 {
            Some(&self.items)
        } else {
            None
        }
    }

    pub fn new_with_one_item(item: StepItem) -> Self {
        StepItems { items: vec![item] }
    }
}

fn validate_refs_single(
    id: EntityId,
    item: &StepItem,
    item_map: &StepItemMap,
) -> Result<(), StepItemMapError> {
    match item.validate_refs(item_map) {
        Ok(_) => Ok(()),
        Err(e) => Err(StepItemMapError::ConvertPart { id, source: e }),
    }
}

/// 参照idの確認
/// 参照先のidが要件を満たしているかどうかを確認する
fn validate_references(item_map: &StepItemMap) -> Result<(), StepItemMapError> {
    for (id, items) in item_map {
        for item in &items.items {
            validate_refs_single(*id, item, item_map)?;
        }
    }
    Ok(())
}

fn convert_step_item(
    ent: SimpleEntity,
    id: EntityId,
) -> Result<Option<StepItem>, StepItemMapError> {
    let keyword = ent.keyword.clone();
    match StepItem::try_from(ent) {
        Ok(item) => Ok(Some(item)),
        Err(ConversionStepItemError::Unsupported(_)) => {
            println!(
                "Skipping unsupported keyword `{:?}` in entity #{}",
                keyword, id
            );
            Ok(None)
        }
        Err(e) => Err(StepItemMapError::ConvertPart { id, source: e }),
    }
}

/// Convert a vector of `StepEntity` (DATA section) into a `StepItemMap`.
/// Complex entities result in multiple `StepItem`s under the same id.
/// * Unsupported keywords are **silently skipped** (they remain unparsed).
/// * Any other conversion error aborts the whole process.
pub fn to_step_item_map(src: Vec<StepEntity>) -> Result<StepItemMap, StepItemMapError> {
    let mut map: StepItemMap = HashMap::with_capacity(src.len());

    for ent in src {
        if map.contains_key(&ent.id) {
            return Err(StepItemMapError::DuplicateId(ent.id));
        }

        let mut skip_flag = false;
        let mut items = Vec::with_capacity(ent.parts.len());
        for part in ent.parts {
            let step_item = convert_step_item(part.clone(), ent.id)?;
            match step_item {
                Some(item) => items.push(item),
                None => {
                    // Unsupported entity, skip it
                    skip_flag = true;
                    break;
                }
            }
        }

        if !skip_flag {
            map.insert(ent.id, StepItems { items });
        }
    }
    // Validate all references in the map
    validate_references(&map)?;
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::{parse_step_entity, StepEntityParseError};

    #[test]
    fn to_step_item_map_simple() {
        let src = vec![
            "#1 = DIRECTION('', (1.0, 2.0, 3.0));",
            "#2 = DIRECTION('', (4.0, 5.0, 6.0));",
            "#3 = SOME_UNSUPPORTED_ENTITY();",
        ];

        let entities: Result<Vec<StepEntity>, StepEntityParseError> =
            src.into_iter().map(parse_step_entity).collect();
        let item_map = to_step_item_map(entities.unwrap()).unwrap();

        assert_eq!(item_map.len(), 2);
        assert!(item_map.contains_key(&1));
        assert!(item_map.contains_key(&2));
        assert!(!item_map.contains_key(&3)); // Unsupported entity is skipped
    }

    #[test]
    fn to_step_item_map_duplicate_id() {
        let src = vec![
            "#1 = DIRECTION('', (1.0, 2.0, 3.0));",
            "#1 = DIRECTION('', (4.0, 5.0, 6.0));", // Duplicate ID
        ];

        let entities: Result<Vec<StepEntity>, StepEntityParseError> =
            src.into_iter().map(parse_step_entity).collect();
        let result = to_step_item_map(entities.unwrap());

        assert!(result.is_err());
        if let Err(StepItemMapError::DuplicateId(id)) = result {
            assert_eq!(id, 1);
        } else {
            panic!("Expected DuplicateId error");
        }
    }

    #[test]
    fn to_step_item_map_conversion_error() {
        let src = vec![
            "#1 = DIRECTION('', (1.0, 2.0, 3.0));",
            "#2 = DIRECTION('', (4.0, 5.0, 6.0, 7.0));", // Invalid part
        ];
        let entities: Result<Vec<StepEntity>, StepEntityParseError> =
            src.into_iter().map(parse_step_entity).collect();
        let entities = entities.unwrap();
        let result = to_step_item_map(entities);
        assert!(result.is_err());
        if let Err(StepItemMapError::ConvertPart { id, source }) = result {
            assert_eq!(id, 2);
            assert!(matches!(source, ConversionStepItemError::ItemCount { .. }));
        } else {
            panic!("Expected ConvertPart error");
        }
    }

    #[test]
    fn to_step_item_map_validate_references() {
        let src = vec![
            "#1 = DIRECTION('', (1.0, 2.0, 3.0));",
            "#2 = DIRECTION('', (4.0, 5.0, 6.0));",
            "#3 = VECTOR('', #1, 2.0);", // Valid reference
        ];

        let entities: Result<Vec<StepEntity>, StepEntityParseError> =
            src.into_iter().map(parse_step_entity).collect();
        let entities = entities.unwrap();
        let result = to_step_item_map(entities);
        assert!(result.is_ok());
        let item_map = result.unwrap();
        assert_eq!(item_map.len(), 3);
        assert!(item_map.contains_key(&1));
        assert!(item_map.contains_key(&2));
        assert!(item_map.contains_key(&3));
    }

    #[test]
    fn to_step_item_map_invalid_reference() {
        let src = vec![
            "#1 = DIRECTION('', (1.0, 2.0, 3.0));",
            "#2 = VECTOR('', #999, 2.0);", // Invalid reference
        ];

        let entities: Result<Vec<StepEntity>, StepEntityParseError> =
            src.into_iter().map(parse_step_entity).collect();
        let entities = entities.unwrap();
        let result = to_step_item_map(entities);
        assert!(result.is_err());
        if let Err(StepItemMapError::ConvertPart { id, source }) = result {
            assert_eq!(id, 2);
            assert!(matches!(
                source,
                ConversionStepItemError::UnresolvedRef { .. }
            ));
        } else {
            panic!("Expected ConvertPart error for unresolved reference");
        }
    }
}

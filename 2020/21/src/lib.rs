use std::collections::{HashMap, HashSet, BTreeMap};

use lazy_static::lazy_static;
use regex::Regex;

pub type Ingredient = String;
pub type Allergen = String;

pub struct Food {
    pub ingredients: HashSet<Ingredient>,
    pub allergens: HashSet<Allergen>,
}

impl std::str::FromStr for Food {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref REGEX: Regex =
                Regex::new(r"^(?P<ingredients>.*).*\(contains (?P<allergens>.*)\)\s*$").unwrap();
            static ref ALLERGEN_PARSE: Regex = Regex::new(r"(\w+)").unwrap();
        }
        let caps = REGEX.captures(s).unwrap();
        let ingredients = caps.name("ingredients").unwrap().as_str();
        let allergens = caps.name("allergens").unwrap().as_str();
        let ingredients = ingredients.split_whitespace().map(str::to_string).collect();
        let allergens = ALLERGEN_PARSE
            .captures_iter(allergens)
            .map(|c| c.get(0).unwrap().as_str().to_string())
            .collect();
        Ok(Self {
            ingredients,
            allergens,
        })
    }
}

pub fn q1<'a>(strings: impl IntoIterator<Item = &'a str>) -> (usize, BTreeMap<Allergen, Ingredient>) {
    use std::collections::hash_map::Entry as MapEntry;
    let mut undetermined: HashMap<Ingredient, usize> = HashMap::new();
    let mut allergen_to_ingredients: HashMap<Allergen, HashSet<Ingredient>> = HashMap::new();
    let mut dangerous_ingredients : BTreeMap<Allergen, Ingredient> = BTreeMap::new();
    for food in strings.into_iter().map(|s| s.parse::<Food>().unwrap()) {
        let allergens = food.allergens;
        let ingredients = food.ingredients;
        for allergen in allergens {
            match allergen_to_ingredients.entry(allergen) {
                MapEntry::Occupied(mut item) => {
                    let ingredient_set = item.get_mut();
                    *ingredient_set = ingredient_set.intersection(&ingredients).cloned().collect();
                }
                MapEntry::Vacant(v) => {
                    v.insert(ingredients.clone());
                }
            }
        }
        for ingredient in ingredients {
            *undetermined.entry(ingredient).or_insert(0) += 1;
        }
    }
    while !allergen_to_ingredients.is_empty() {
        let (allergen, _) = allergen_to_ingredients
            .iter()
            .find(|(_allergen, ingredients)| ingredients.len() == 1)
            .unwrap();
        let allergen = allergen.clone();
        let (allergen, ingredient) = allergen_to_ingredients.remove_entry(&allergen).unwrap();
        let ingredient = ingredient.into_iter().next().unwrap();
        for i_set in allergen_to_ingredients.values_mut() {
            i_set.remove(&ingredient);
        }
        undetermined.remove(&ingredient);
        dangerous_ingredients.insert(allergen, ingredient);
    }
    return (undetermined.values().sum(), dangerous_ingredients);
}

#[cfg(test)]
mod test {
    const RECEIPT: &str = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
trh fvjkl sbzzf mxmxvkd (contains dairy)
sqjhc fvjkl (contains soy)
sqjhc mxmxvkd sbzzf (contains fish)";
    use super::*;
    #[test]
    fn test_parse_food() {
        let s = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)";
        let food = s.parse::<Food>().unwrap();
        assert_eq!(
            food.ingredients,
            "mxmxvkd kfcds sqjhc nhms"
                .split_whitespace()
                .map(str::to_string)
                .collect()
        );
        assert_eq!(
            food.allergens,
            ["dairy", "fish"]
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        )
    }
    #[test]
    fn test_q1() {
        assert_eq!(q1(RECEIPT.lines()), 5);
    }
}

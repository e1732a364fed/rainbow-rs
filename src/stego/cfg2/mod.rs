use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct FinalProduction {
    text: String,

    product_type: ProductType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductType {
    Plain,
    VariableName,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CFG {
    variables: HashMap<String, Vec<FinalProduction>>,
}

impl CFG {
    fn expand(&self, text: &str, choices: Option<&HashMap<String, usize>>) -> String {
        let mut result = text.to_string();
        while result.contains('{') {
            let start = result.find('{').unwrap();
            let end = result.find('}').unwrap();
            let var_name = &result[start + 1..end];

            let productions = self.variables.get(var_name).unwrap();
            // Use the provided index if available, otherwise use first production
            let production = if let Some(choice_map) = choices {
                let index = choice_map.get(var_name).unwrap_or(&0);
                &productions[*index]
            } else {
                &productions[0]
            };

            result.replace_range(start..=end, &production.text);
        }
        result
    }
}
#[cfg(test)]
mod test {
    use common_macros::hash_map;
    use std::collections::HashMap;

    use crate::stego::cfg2::CFG;

    use super::{FinalProduction, ProductType};
    #[test]
    fn test() {
        // let terminals = HashMap::

        let vp = vec![
            FinalProduction {
                text: "went ﬁshing {where}".to_string(),
                product_type: ProductType::Replace,
            },
            FinalProduction {
                text: "went bowling {where}".to_string(),
                product_type: ProductType::Replace,
            },
        ];

        let wp = vec![
            FinalProduction {
                text: "in {direction} Iowa.".to_string(),
                product_type: ProductType::Replace,
            },
            FinalProduction {
                text: "in {direction} Minnesota.".to_string(),
                product_type: ProductType::Replace,
            },
        ];

        let dp = vec![
            FinalProduction {
                text: "northern".to_string(),
                product_type: ProductType::Plain,
            },
            FinalProduction {
                text: "southern".to_string(),
                product_type: ProductType::Plain,
            },
        ];

        let np = vec![
            FinalProduction {
                text: "Fred".to_string(),
                product_type: ProductType::Plain,
            },
            FinalProduction {
                text: "Barney".to_string(),
                product_type: ProductType::Plain,
            },
        ];

        let start = vec![FinalProduction {
            text: "{noun} {verb}".to_string(),
            product_type: ProductType::Replace,
        }];

        let variables = hash_map! {
            "start".to_owned() =>  start  ,
            "noun".to_owned() =>  np  ,
            "verb".to_owned() =>  vp  ,
            "where".to_owned() =>  wp  ,
            "direction".to_owned() =>  dp  ,
        };

        // Create CFG instance
        let cfg = CFG { variables };

        // Test case 1: No choices (default behavior)
        let result = cfg.expand("{start}", None);
        assert_eq!(result, "Fred went ﬁshing in northern Iowa.");

        // Test case 2: All choices specified
        let choices1 = hash_map! {
            "noun".to_owned() => 1,      // Choose "Barney"
            "verb".to_owned() => 1,      // Choose "went bowling"
            "where".to_owned() => 1,     // Choose "in {direction} Minnesota"
            "direction".to_owned() => 1,  // Choose "southern"
        };
        let result1 = cfg.expand("{start}", Some(&choices1));
        assert_eq!(result1, "Barney went bowling in southern Minnesota.");

        // Test case 3: Partial choices (missing some variables)
        let choices2 = hash_map! {
            "noun".to_owned() => 1,      // Choose "Barney"
            "verb".to_owned() => 1,      // Choose "went bowling"
            // "where" and "direction" not specified, should use default (index 0)
        };
        let result2 = cfg.expand("{start}", Some(&choices2));
        assert_eq!(result2, "Barney went bowling in northern Iowa.");

        // Test case 4: Different combination of choices
        let choices3 = hash_map! {
            "noun".to_owned() => 0,      // Choose "Fred"
            "verb".to_owned() => 1,      // Choose "went bowling"
            "where".to_owned() => 1,     // Choose "in {direction} Minnesota"
            "direction".to_owned() => 0,  // Choose "northern"
        };
        let result3 = cfg.expand("{start}", Some(&choices3));
        assert_eq!(result3, "Fred went bowling in northern Minnesota.");
    }
}

/*!
 * mod cfg provides a [`CFG`] that can be used to generate and decode data using a context-free grammar (CFG).
 *
 *
 */

use std::collections::{BTreeMap, HashMap};

use bytes::{BufMut, BytesMut};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::utils::find_matching_brace;
use crate::{stego::Encoder, RainbowError, Result};

use fake::{faker::company::en::*, faker::name::en::*, Fake};

/// 约定 production 的起点记为 `{start}`
pub const START_TAG: &str = "{start}";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFG {
    /// Production 是 CFG 的产生式, 其内容键为变量名，值为该变量的所有可能的产生式
    ///
    /// "产生式"要为类似 `"some text  {variable_name} another text"` 的形式
    ///
    /// 如果没有大括号，则为 Terminal, 大括号内的内容为变量名
    ///
    /// 因为 需要以固定的顺序遍历，所以使用 BTreeMap
    ///
    /// 其中起点这里约定 记为 `{start}`
    production: BTreeMap<String, Vec<String>>,
}

impl CFG {
    /// expand 函数用于将 CFG 的`{}`变量替换为具体的值. 它将会进行多次替换，直到所有变量都被替换为止
    ///
    /// 若 choices 为 None，则使用所有变量的第一个选择
    ///
    /// 若 choices 为 Some，则使用给出的选择
    ///
    /// 若 choices 中对于某个变量没有给出选择，则使用第一个选择
    pub fn expand(&self, text: &str, choices: Option<&HashMap<String, usize>>) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find('{') {
            let end = find_matching_brace(&result, start).unwrap();

            let var_name = &result[start + 1..end];
            let productions = self.production.get(var_name).unwrap();

            let production = if let Some(choice_map) = choices {
                let index = choice_map.get(var_name).unwrap_or(&0);
                &productions[*index]
            } else {
                &productions[0]
            };

            result.replace_range(start..=end, production);
        }
        result
    }

    /// 尝试该选择组合是否能生成目标文本
    pub fn match_choices(&self, target_text: &str, choices: &HashMap<String, usize>) -> bool {
        self.expand(START_TAG, Some(choices)) == target_text
    }

    /// 生成所有可能的选择组合
    pub fn generate_all_choices(&self) -> Vec<HashMap<String, usize>> {
        let mut all_choices = vec![HashMap::new()];

        // 对每个变量
        for (var_name, productions) in &self.production {
            let mut new_choices = Vec::new();

            // 对现有的每个选择组合
            for choice_map in all_choices {
                // 对该变量的每个可能的产生式
                for index in 0..productions.len() {
                    let mut new_choice = choice_map.clone();
                    new_choice.insert(var_name.clone(), index);
                    new_choices.push(new_choice);
                }
            }

            all_choices = new_choices;
        }

        all_choices
    }

    /// 由目标文本反推选择组合, 通过尝试每个选择组合的方式。
    ///
    /// bad performance
    pub fn reverse_by_try_all(&self, target_text: &str) -> Option<HashMap<String, usize>> {
        // 生成所有可能的选择组合
        let all_choices = self.generate_all_choices();

        // 尝试每个选择组合
        all_choices
            .into_iter()
            .find(|choices| self.match_choices(target_text, choices))
    }

    /// 由目标文本反推选择组合，通过递归剪枝的方式。
    ///
    /// good performance
    pub fn reverse(&self, target_text: &str) -> Option<HashMap<String, usize>> {
        let mut choices = HashMap::new();
        if self.match_recursive(target_text, START_TAG, &mut choices, false) {
            Some(choices)
        } else {
            None
        }
    }

    /// 类似reverse， target_text 会以 所返回的选择组合 所 expand 后的结果为开头
    pub fn reverse_by_start_with(&self, target_text: &str) -> Option<HashMap<String, usize>> {
        let mut choices = HashMap::new();
        if self.match_recursive(target_text, START_TAG, &mut choices, true) {
            Some(choices)
        } else {
            None
        }
    }

    fn match_recursive(
        &self,
        target: &str,
        pattern: &str,
        choices: &mut HashMap<String, usize>,
        by_start_with: bool,
    ) -> bool {
        match pattern.find("{") {
            None => {
                if by_start_with {
                    target.starts_with(pattern)
                } else {
                    target == pattern
                }
            }
            Some(start) => {
                let end = find_matching_brace(pattern, start).unwrap();
                let var_name = &pattern[start + 1..end];

                if let Some(productions) = self.production.get(var_name) {
                    for (index, production) in productions.iter().enumerate() {
                        choices.insert(var_name.to_string(), index);

                        let mut new_pattern = pattern.to_string();

                        new_pattern.replace_range(start..=end, production);

                        if let Some(bracket_start) = new_pattern.find('{') {
                            if bracket_start != 0
                                && !target.starts_with(&new_pattern[..bracket_start])
                            {
                                continue;
                            }
                        }

                        if self.match_recursive(target, &new_pattern, choices, by_start_with) {
                            return true;
                        }

                        choices.remove(var_name);
                    }
                }

                false
            }
        }
    }

    /// 将选择映射转换成字节数据
    pub fn choices_to_bytes(&self, choices: &HashMap<String, usize>) -> Vec<u8> {
        let mut result = Vec::new();
        let mut current_byte = 0u8;
        let mut bits_in_current_byte = 0;

        // 对每个变量
        for (var_name, productions) in &self.production {
            let num_productions = productions.len();
            if num_productions <= 1 {
                continue;
            }

            let bits_per_var = (num_productions as f64).log2().floor() as u8;
            if bits_per_var == 0 {
                continue;
            }

            let choice = choices.get(var_name).unwrap_or(&0);
            let value = (*choice as u8) % num_productions as u8;

            // 从高位到低位处理每个比特
            for bit_pos in (0..bits_per_var).rev() {
                let bit = (value >> bit_pos) & 1;
                current_byte = (current_byte << 1) | bit;
                bits_in_current_byte += 1;

                // 当积累了8位时，将字节添加到结果中
                if bits_in_current_byte == 8 {
                    result.push(current_byte);
                    current_byte = 0;
                    bits_in_current_byte = 0;
                }
            }
        }

        // 处理最后可能不完整的字节
        if bits_in_current_byte > 0 {
            current_byte <<= 8 - bits_in_current_byte;
            result.push(current_byte);
        }

        result
    }

    // 添加一个新的辅助方法来计算单个句子的容量
    pub fn bits_capacity(&self) -> usize {
        self.production
            .values()
            .map(|productions| {
                let num_productions = productions.len();
                if num_productions <= 1 {
                    0
                } else {
                    (num_productions as f64).log2().floor() as usize
                }
            })
            .sum()
    }
}

const VERBS: [&str; 16] = [
    "reveals",
    "launches",
    "discovers",
    "introduces",
    "develops",
    "implements",
    "demonstrates",
    "unveils",
    "presents",
    "confirms",
    "establishes",
    "initiates",
    "validates",
    "showcases",
    "releases",
    "publishes",
];

impl CFG {
    /// capacity: 4 bits
    pub fn example1() -> CFG {
        let vp = vec![
            "went ﬁshing {where}".to_string(),
            "went bowling {where}".to_string(),
        ];

        let wp = vec![
            "in {direction} Iowa.".to_string(),
            "in {direction} Minnesota.".to_string(),
        ];

        let dp = vec!["northern".to_string(), "southern".to_string()];

        let np = vec!["Fred".to_string(), "Barney".to_string()];

        let start = vec!["{noun} {verb}".to_string()];

        let variables = b_tree_map! {
            "start".to_owned() =>  start  ,
            "noun".to_owned() =>  np  ,
            "verb".to_owned() =>  vp  ,
            "where".to_owned() =>  wp  ,
            "direction".to_owned() =>  dp  ,
        };

        // Create CFG instance
        let cfg = CFG {
            production: variables,
        };

        println!("capacity: {}", cfg.bits_capacity());

        cfg
    }

    /// capacity: 32 bits
    pub fn example2() -> CFG {
        let start =
            vec!["{SUBJECT_VERB_OBJECT}\n{DATELINE}\n{CONTENT}\n{QUOTE_INTRO} {QUOTE}".to_string()];

        let svb = vec!["{SUBJECT} {VERB} {OBJECT}".to_string()];

        // 主语
        const SUBJECTS: [&str; 32] = [
            "Tech giant",
            "Local authorities",
            "Scientists",
            "Researchers",
            "Industry experts",
            "Market analysts",
            "Government officials",
            "Medical professionals",
            "Environmental activists",
            "Security researchers",
            "Financial experts",
            "Education leaders",
            "Technology pioneers",
            "Healthcare providers",
            "Climate scientists",
            "Policy makers",
            "Business leaders",
            "Innovation experts",
            "Data scientists",
            "AI researchers",
            "Cybersecurity experts",
            "Space scientists",
            "Marine biologists",
            "Energy researchers",
            "Quantum physicists",
            "Neuroscientists",
            "Biotechnology firms",
            "Software developers",
            "Automotive engineers",
            "Aerospace experts",
            "Economic analysts",
            "Urban planners",
        ];

        // 宾语
        const OBJECTS: [&str; 32] = [
            "new findings",
            "innovative solution",
            "major development",
            "groundbreaking research",
            "revolutionary platform",
            "cutting-edge system",
            "advanced framework",
            "sustainable initiative",
            "strategic partnership",
            "quantum breakthrough",
            "AI-powered solution",
            "digital transformation",
            "research findings",
            "technological advancement",
            "innovative approach",
            "sustainable solution",
            "security protocol",
            "efficiency improvement",
            "market strategy",
            "development framework",
            "research methodology",
            "optimization technique",
            "implementation strategy",
            "analytical tool",
            "prediction model",
            "automation system",
            "integration platform",
            "monitoring system",
            "validation process",
            "enhancement protocol",
            "deployment strategy",
            "scaling solution",
        ];

        const CITIES: [&str; 32] = [
            "NEW YORK",
            "LONDON",
            "TOKYO",
            "BEIJING",
            "SAN FRANCISCO",
            "SINGAPORE",
            "BERLIN",
            "PARIS",
            "SEOUL",
            "SYDNEY",
            "DUBAI",
            "TORONTO",
            "SHANGHAI",
            "MUMBAI",
            "AMSTERDAM",
            "STOCKHOLM",
            "HONG KONG",
            "BOSTON",
            "TEL AVIV",
            "ZURICH",
            "SEATTLE",
            "AUSTIN",
            "BANGALORE",
            "MUNICH",
            "VANCOUVER",
            "COPENHAGEN",
            "OSLO",
            "VIENNA",
            "MELBOURNE",
            "MONTREAL",
            "GENEVA",
            "HELSINKI",
        ];

        const CONTENT: [&str; 8] = [
            "This breakthrough could revolutionize the industry. ",
            "The development marks a significant milestone in the field. ",
            "The innovation represents a major leap forward in technology. ",
            "This discovery opens up new possibilities for future research. ",
            "The findings suggest a paradigm shift in the industry. ",
            "The results demonstrate unprecedented potential for growth. ",
            "This advancement challenges existing technological limitations. ",
            "The research reveals promising applications across sectors. ",
        ];

        const QUOTES: [&str; 8] = [
            "\"This is just the beginning of a new era in technology.\"",
            "\"Our findings will transform the way we approach this field.\"",
            "\"The implications of this discovery are far-reaching.\"",
            "\"We're excited about the potential applications of this breakthrough.\"",
            "\"This development represents a quantum leap in our capabilities.\"",
            "\"The results have exceeded our most optimistic expectations.\"",
            "\"This marks a pivotal moment in our research journey.\"",
            "\"We're just scratching the surface of what's possible.\"",
        ];

        const QUOTE_INTROS: [&str; 4] = [
            "The lead scientist stated,",
            "The project director commented,",
            "The research team leader noted,",
            "The chief investigator remarked,",
        ];

        let dates: Vec<String> = (1..=32)
            .map(|day| format!("January {}, 2024", day))
            .collect();

        let dates: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();

        let variables = b_tree_map! {
            "start".to_owned() =>  start,
            "SUBJECT_VERB_OBJECT".to_owned() =>  svb,
            "SUBJECT".to_owned() =>  init_plain_by_list(&SUBJECTS),
            "VERB".to_owned() =>  init_plain_by_list(&VERBS),
            "OBJECT".to_owned() =>  init_plain_by_list(&OBJECTS),
            "CITY".to_owned() =>  init_plain_by_list(&CITIES),
            "DATE".to_owned() =>  init_plain_by_list(&dates),
            "QUOTE".to_owned() =>  init_plain_by_list(&QUOTES),
            "QUOTE_INTRO".to_owned() =>  init_plain_by_list(&QUOTE_INTROS),
            "CONTENT".to_owned() =>  init_plain_by_list(&CONTENT),
            "DATELINE".to_owned() =>  init_plain_by_list(& ["{DATE} {CITY}"]),
        };

        // Create CFG instance
        let cfg = CFG {
            production: variables,
        };

        println!("capacity: {}", cfg.bits_capacity());

        assert_eq!(cfg.bits_capacity(), 32);

        cfg
    }
}
use super::Random;

impl Random for CFG {
    /// capacity: 32 bits
    fn random() -> Self {
        let start = vec!["{HEADLINE}\n{DATELINE}\n{CONTENT}\n{QUOTE_INTRO} {QUOTE}".to_string()];

        let headline = vec!["{SUBJECT} {VERB} {OBJECT}".to_string()];

        // 生成随机主语 (32)
        let subjects: Vec<String> = (0..8)
            .flat_map(|_| {
                let company = CompanyName().fake::<String>();
                vec![
                    company.clone(),
                    format!("The {} team", company),
                    format!("{} researchers", company),
                    format!("{} executives", company),
                ]
            })
            .collect();

        assert_eq!(subjects.len(), 32);

        // 生成随机宾语 (32个)
        let objects: Vec<String> = {
            let adj = [
                "innovative",
                "groundbreaking",
                "revolutionary",
                "advanced",
                "cutting-edge",
                "state-of-the-art",
                "next-generation",
                "intelligent",
            ];
            let noun = [
                "technology",
                "solution",
                "platform",
                "system",
                "framework",
                "architecture",
                "infrastructure",
                "ecosystem",
            ];
            let mut combinations: Vec<String> = adj
                .iter()
                .flat_map(|a| noun.iter().map(move |n| format!("{} {}", a, n)))
                .collect();
            assert_eq!(combinations.len(), 64); // 8x8=64 种组合
            combinations.shuffle(&mut rand::thread_rng());
            combinations.into_iter().take(32).collect()
        };

        assert_eq!(objects.len(), 32);

        // 生成随机城市 (32)
        let cities: Vec<String> = {
            let mut cities = Vec::new();
            let mut seen = std::collections::HashSet::new();

            while cities.len() < 32 {
                let city = fake::faker::address::en::CityName()
                    .fake::<String>()
                    .to_uppercase();

                if seen.insert(city.clone()) {
                    cities.push(city);
                }
            }

            cities.shuffle(&mut rand::thread_rng());
            cities
        };

        assert_eq!(cities.len(), 32);

        // 生成随机内容段落 (8个)
        let content: Vec<String> = {
            let templates = [
                "The development marks {} in the industry. ",
                "This breakthrough could {} across sectors. ",
                "The innovation represents {} for future growth. ",
                "The technology demonstrates {} in real-world applications. ",
            ];
            let impacts = [
                "a significant milestone",
                "revolutionary potential",
                "unprecedented opportunities",
                "remarkable capabilities",
            ];
            let mut combinations: Vec<String> = templates
                .iter()
                .flat_map(|t| impacts.iter().map(move |i| t.replace("{}", i).to_string()))
                .collect();
            combinations.shuffle(&mut rand::thread_rng());
            combinations.into_iter().take(8).collect()
        };

        assert_eq!(content.len(), 8);

        // 生成随机引述 (8个)
        let quotes: Vec<String> = {
            let names: Vec<String> = (0..16).map(|_| Name().fake::<String>()).collect();
            let titles: Vec<String> = (0..16)
                .map(|_| fake::faker::job::en::Title().fake::<String>())
                .collect();
            let templates = [
                "\"We are excited about the potential impact of this development,\" said {}, {}.",
                "\"This represents a major step forward for our industry,\" commented {}, {}.",
                "\"The implications of this breakthrough are far-reaching,\" stated {}, {}.",
                "\"This is just the beginning of our journey,\" noted {}, {}.",
                "\"Our findings exceed our most optimistic expectations,\" remarked {}, {}.",
                "\"This development opens up new possibilities,\" explained {}, {}.",
                "\"We're seeing unprecedented results in our tests,\" reported {}, {}.",
                "\"This innovation will transform the industry,\" declared {}, {}.",
            ];
            let mut combinations: Vec<String> = templates
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    t.replace("{}", &names[i * 2])
                        .replace("{}", &titles[i * 2])
                        .to_string()
                })
                .collect();
            combinations.shuffle(&mut rand::thread_rng());
            combinations
        };

        assert_eq!(quotes.len(), 8);

        // 生成随机引述介绍 (4个)
        let quote_intros: Vec<String> = {
            let positions: Vec<String> = (0..8)
                .map(|_| fake::faker::job::en::Title().fake::<String>())
                .collect();
            let verbs = [
                "stated",
                "commented",
                "noted",
                "remarked",
                "explained",
                "emphasized",
                "observed",
                "added",
            ];
            let mut combinations: Vec<String> = positions
                .iter()
                .flat_map(|p| verbs.iter().map(move |v| format!("The {} {},", p, v)))
                .collect();
            combinations.shuffle(&mut rand::thread_rng());
            combinations.into_iter().take(4).collect()
        };

        assert_eq!(quote_intros.len(), 4);

        let dates2 = (1..=32)
            .map(|d| format!("January {}, 2024", d))
            .collect::<Vec<_>>();
        let dates = dates2.iter().map(AsRef::as_ref).collect::<Vec<_>>();

        assert_eq!(dates.len(), 32);

        let variables = b_tree_map! {
            "start".to_owned() => start,
            "HEADLINE".to_owned() => headline,
            "SUBJECT".to_owned() => init_plain_by_list(&subjects.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "VERB".to_owned() => init_plain_by_list(&VERBS),
            "OBJECT".to_owned() => init_plain_by_list(&objects.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "CITY".to_owned() => init_plain_by_list(&cities.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "DATE".to_owned() => init_plain_by_list(&dates),
            "QUOTE".to_owned() => init_plain_by_list(&quotes.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "QUOTE_INTRO".to_owned() => init_plain_by_list(&quote_intros.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "CONTENT".to_owned() => init_plain_by_list(&content.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            "DATELINE".to_owned() => init_plain_by_list(&["{DATE} {CITY}"]),
        };

        let cfg = CFG {
            production: variables,
        };
        assert_eq!(
            cfg.bits_capacity(),
            32,
            "Random CFG capacity must = 32 bits"
        );
        cfg
    }
}

/// CFGEncoder 包装了 [`CFG`], 使其可以针对任意长度的数据进行编码
#[derive(Debug, Clone)]
pub struct CFGEncoder {
    cfg: CFG,
}

impl Default for CFGEncoder {
    fn default() -> Self {
        Self {
            cfg: CFG::example2(),
        }
    }
}

impl Random for CFGEncoder {
    fn random() -> Self {
        Self { cfg: CFG::random() }
    }
}

impl Encoder for CFGEncoder {
    fn name(&self) -> &'static str {
        "cfg"
    }

    fn get_mime_type(&self) -> &'static str {
        "text/plain"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // 在直接编码时，需要要求 data 的 长度大于 capacity/8 字节

        // 但是，我们在这里通过添加头部的方式来解决

        // 头部 包含 数据长度, 四字节。
        // 如果头部的四字节 还不够 capacity/8 字节，则会在头部后面再 填充 (length - capacity/8) 字节 长度的0

        let data_length = data.len();
        let capacity = self.cfg.bits_capacity();

        let mut bytes_mut = BytesMut::new();
        bytes_mut.put_u32(data_length as u32);

        // 如果头部的四字节 还不够 capacity/8 字节，则会在头部后面再 填充 (length - capacity/8) 字节 长度的0
        if capacity / 8 > 4 {
            let padding_length = capacity / 8 - 4;

            bytes_mut.extend_from_slice(&vec![0; padding_length]);
        }
        bytes_mut.extend_from_slice(data);

        let data = bytes_mut.as_ref();

        let mut result = Vec::new();
        let mut current_byte_index = 0;
        let mut current_bit_index = 0;

        let data_len = data.len();

        while current_byte_index < data.len() {
            let mut choices = HashMap::new();

            // 为当前句子编码数据
            for (var_name, productions) in &self.cfg.production {
                let num_productions = productions.len();
                if num_productions <= 1 {
                    continue;
                }

                let bits_per_var = (num_productions as f64).log2().floor() as u8;
                if bits_per_var == 0 {
                    continue;
                }

                // 检查是否还有足够的数据需要编码
                if current_byte_index >= data_len {
                    break;
                }

                let mut value = 0;
                for (bits_read, _) in (0_u8..).zip(0..bits_per_var) {
                    if current_byte_index >= data_len {
                        // 如果已经读取了部分比特，需要左移剩余的位数
                        if bits_read > 0 {
                            value <<= bits_per_var - bits_read;
                        }
                        break;
                    }

                    let bit = (data[current_byte_index] >> (7 - current_bit_index)) & 1;
                    value = (value << 1) | bit;

                    current_bit_index += 1;
                    if current_bit_index == 8 {
                        current_bit_index = 0;
                        current_byte_index += 1;
                    }
                }

                value %= num_productions as u8;

                choices.insert(var_name.clone(), value as usize);
            }

            // 生成当前句子
            let sentence = self.cfg.expand(START_TAG, Some(&choices));

            // 添加分隔符（如果不是第一个句子）
            if !result.is_empty() {
                result.extend_from_slice(b" ");
            }
            result.extend_from_slice(sentence.as_bytes());
        }

        Ok(result)
    }

    /// 要求 capacity 必须是 2 的幂
    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        let capacity = self.cfg.bits_capacity();
        assert!(capacity.is_power_of_two());

        let cs = String::from_utf8_lossy(content);
        let mut result = Vec::new();

        // 一种需要补全 单个字节内部的情况。
        // 只可能是在 capacity < 8时, 此时只有 2 和 4 两种可能

        // 因为如果 capacity > 8, 那么 capacity 一定是 8 的倍数， 此时生成的字节是完整的

        let mut required_short_fill_count = 0;

        // 处理每个完整的字节
        let mut remaining_text = cs.as_ref();
        while !remaining_text.is_empty() {
            remaining_text = remaining_text.trim_start();

            let choices =
                self.cfg
                    .reverse_by_start_with(remaining_text)
                    .ok_or(RainbowError::InvalidData(
                        "reverse_by_start_with got None".to_string(),
                    ))?;

            let bytes = self.cfg.choices_to_bytes(&choices);

            if capacity < 8 {
                assert!(bytes.len() == 1);
                let this_byte = bytes[0];

                if required_short_fill_count == 0 {
                    required_short_fill_count = 8 / capacity - 1;
                    result.push(this_byte);
                } else {
                    let last_byte = result.last_mut().unwrap();

                    let real_this_byte =
                        this_byte >> (capacity * (8 / capacity - required_short_fill_count));

                    *last_byte += real_this_byte;

                    required_short_fill_count -= 1;
                }
            } else {
                result.extend_from_slice(&bytes);
            }

            let expanded = self.cfg.expand(START_TAG, Some(&choices));
            remaining_text = &remaining_text[expanded.len()..];
        }

        // 参照 encode 的逻辑， 先从头部获取数据长度

        let data_length = u32::from_be_bytes(result.as_slice()[..4].try_into().unwrap()) as usize;

        let capacity_per_byte = capacity / 8;

        let start_at = if capacity_per_byte > 4 {
            capacity_per_byte
        } else {
            4
        };

        if result.len() < start_at + data_length {
            return Err(RainbowError::LengthMismatch(
                result.len(),
                start_at + data_length,
                "result.len() < start_at + data_length".to_string(),
            ));
        }

        let result = &result[start_at..start_at + data_length];

        Ok(result.to_vec())
    }
}

pub fn init_plain_by_list(list: &[&str]) -> Vec<String> {
    list.iter().map(|t| t.to_string()).collect()
}

use common_macros::b_tree_map;

#[cfg(test)]
mod test {
    use super::CFGEncoder;
    use crate::stego::{
        cfg::{CFG, START_TAG},
        Encoder, Random,
    };
    use common_macros::hash_map;
    use rand::Rng;

    #[test]
    fn test() {
        let cfg = CFG::example1();

        // Test case 1: No choices (default behavior)
        let result = cfg.expand(START_TAG, None);
        assert_eq!(result, "Fred went ﬁshing in northern Iowa.");

        // Test case 2: All choices specified
        let choices1 = hash_map! {
            "noun".to_owned() => 1,      // Choose "Barney"
            "verb".to_owned() => 1,      // Choose "went bowling"
            "where".to_owned() => 1,     // Choose "in {direction} Minnesota"
            "direction".to_owned() => 1,  // Choose "southern"
        };
        let result1 = cfg.expand(START_TAG, Some(&choices1));
        assert_eq!(result1, "Barney went bowling in southern Minnesota.");

        // Test case 3: Partial choices (missing some variables)
        let choices2 = hash_map! {
            "noun".to_owned() => 1,      // Choose "Barney"
            "verb".to_owned() => 1,      // Choose "went bowling"
            // "where" and "direction" not specified, should use default (index 0)
        };
        let result2 = cfg.expand(START_TAG, Some(&choices2));
        assert_eq!(result2, "Barney went bowling in northern Iowa.");

        // Test case 4: Different combination of choices
        let choices3 = hash_map! {
            "noun".to_owned() => 0,      // Choose "Fred"
            "verb".to_owned() => 1,      // Choose "went bowling"
            "where".to_owned() => 1,     // Choose "in {direction} Minnesota"
            "direction".to_owned() => 0,  // Choose "northern"
        };
        let result3 = cfg.expand(START_TAG, Some(&choices3));
        assert_eq!(result3, "Fred went bowling in northern Minnesota.");
    }

    #[test]
    fn test_reverse() {
        let cfg = CFG::example1();

        println!("all choices {:#?}", cfg.generate_all_choices());

        // Test case 1: Simple reverse
        let text = "Fred went ﬁshing in northern Iowa.";
        let choices = cfg.reverse_by_try_all(text);
        assert!(choices.is_some());
        let choices = choices.unwrap();
        assert_eq!(*choices.get("noun").unwrap(), 0);
        assert_eq!(*choices.get("verb").unwrap(), 0);
        assert_eq!(*choices.get("where").unwrap(), 0);
        assert_eq!(*choices.get("direction").unwrap(), 0);

        // 验证反向结果
        let expanded = cfg.expand(START_TAG, Some(&choices));
        assert_eq!(expanded, text);

        // Test case 2: Different combination
        let text2 = "Barney went bowling in southern Minnesota.";
        let choices2 = cfg.reverse_by_try_all(text2);
        assert!(choices2.is_some());
        let choices2 = choices2.unwrap();
        assert_eq!(*choices2.get("noun").unwrap(), 1);
        assert_eq!(*choices2.get("verb").unwrap(), 1);
        assert_eq!(*choices2.get("where").unwrap(), 1);
        assert_eq!(*choices2.get("direction").unwrap(), 1);

        // 验证反向结果
        let expanded2 = cfg.expand(START_TAG, Some(&choices2));
        assert_eq!(expanded2, text2);

        // Test case 3: Invalid text should return None
        let invalid_text = "Invalid text that doesn't match grammar";
        assert!(cfg.reverse_by_try_all(invalid_text).is_none());
    }

    #[test]
    fn test_reverse_optimized() {
        let cfg = CFG::example1();

        // 测试用例1：基本匹配
        let text1 = "Fred went ﬁshing in northern Iowa.";
        let choices1 = cfg.reverse(text1);
        assert!(choices1.is_some());
        let choices1 = choices1.unwrap();
        assert_eq!(*choices1.get("noun").unwrap(), 0); // Fred
        assert_eq!(*choices1.get("verb").unwrap(), 0); // went fishing
        assert_eq!(*choices1.get("where").unwrap(), 0); // in {direction} Iowa
        assert_eq!(*choices1.get("direction").unwrap(), 0); // northern

        println!("choices 1 basic ok, {:#?}", choices1);

        // 验证展开结果
        let expanded1 = cfg.expand(START_TAG, Some(&choices1));
        assert_eq!(expanded1, text1);

        // 测试用例2：所有变量都选择第二个选项
        let text2 = "Barney went bowling in southern Minnesota.";
        let choices2 = cfg.reverse(text2);
        assert!(choices2.is_some());
        let choices2 = choices2.unwrap();
        assert_eq!(*choices2.get("noun").unwrap(), 1); // Barney
        assert_eq!(*choices2.get("verb").unwrap(), 1); // went bowling
        assert_eq!(*choices2.get("where").unwrap(), 1); // in {direction} Minnesota
        assert_eq!(*choices2.get("direction").unwrap(), 1); // southern

        println!("choices 2 all ok, {:#?}", choices2);

        // 验证展开结果
        let expanded2 = cfg.expand(START_TAG, Some(&choices2));
        assert_eq!(expanded2, text2);

        // 测试用例3：混合选择
        let text3 = "Fred went bowling in southern Iowa.";
        let choices3 = cfg.reverse(text3);
        assert!(choices3.is_some());
        let choices3 = choices3.unwrap();
        assert_eq!(*choices3.get("noun").unwrap(), 0); // Fred
        assert_eq!(*choices3.get("verb").unwrap(), 1); // went bowling
        assert_eq!(*choices3.get("where").unwrap(), 0); // in {direction} Iowa
        assert_eq!(*choices3.get("direction").unwrap(), 1); // southern

        println!("choices 3 mixed ok, {:#?}", choices3);

        // 验证展开结果
        let expanded3 = cfg.expand(START_TAG, Some(&choices3));
        assert_eq!(expanded3, text3);

        // 测试用例4：无效输入
        let invalid_text = "This text doesn't match any production";
        let invalid_result = cfg.reverse(invalid_text);
        assert!(invalid_result.is_none());

        println!("choices 4 invalid ok, {:#?}", invalid_result);

        // 测试用例5：空字符串
        let empty_text = "";
        let empty_result = cfg.reverse(empty_text);
        assert!(empty_result.is_none());

        println!("choices 5 empty ok, {:#?}", empty_result);
    }

    #[test]
    fn test_encode_decode1() {
        let cfg = CFG::example1();
        test_encode_decode_for_cfg(cfg);
    }

    #[test]
    fn test2() {
        let cfg = CFG::example2();
        test_encode_decode_for_cfg(cfg);
    }

    #[test]
    fn test_random_cfg() {
        let cfg = CFG::random();
        test_encode_decode_for_cfg(cfg);
    }

    #[test]
    fn test_encode_decode_large1() {
        let cfg = CFG::example1();
        test_encode_decode_large_for_cfg(cfg);
    }

    #[test]
    fn test_encode_decode_large2() {
        let cfg = CFG::example2();
        test_encode_decode_large_for_cfg(cfg);
    }

    fn test_encode_decode_for_cfg(cfg: CFG) {
        let encoder = CFGEncoder { cfg };

        // Test case 0: Empty data
        let empty_data = vec![];
        let encoded = encoder.encode(&empty_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(empty_data, decoded);

        println!("test0 ok");

        // Test case 1: Basic encoding and decoding
        let data = vec![0b10101010];
        let encoded = encoder.encode(&data).unwrap();

        println!("encoded\n{}", String::from_utf8_lossy(&encoded));
        let decoded = encoder.decode(&encoded).unwrap();

        println!("decoded {:?}", decoded);
        assert_eq!(data, decoded);

        println!("test1 ok");

        // Test case 3: Multiple bytes
        let multi_bytes = vec![0xFF, 0x00];
        let encoded = encoder.encode(&multi_bytes).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(multi_bytes, decoded);

        println!("encoded {}", String::from_utf8_lossy(&encoded));
        println!("decoded {:?}", decoded);

        println!("test3 ok");

        // Test case 3.2: Multiple bytes
        let multi_bytes = vec![0xFF, 0x00, 0xAA];
        let encoded = encoder.encode(&multi_bytes).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(multi_bytes, decoded);

        println!("test3.2 ok");

        // Test case 4: Large data
        let large_data: Vec<u8> = (0..32).collect();
        let encoded = encoder.encode(&large_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(large_data, decoded);

        println!("test4 ok");

        // Test case 5: Random data
        let mut rng = rand::thread_rng();
        let random_data: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        let encoded = encoder.encode(&random_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(random_data, decoded);

        println!("test5 ok");

        let test_data = b"Hello, MIME Type Steganography!!!"; //33 bytes
        let encoded = encoder.encode(&test_data[..]).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(test_data[..], decoded);
        println!("test6 ok");

        let test_data = b"Hello, MIME Type Steganography!!"; //32 bytes
        let encoded = encoder.encode(&test_data[..]).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(test_data[..], decoded);
        println!("test7 ok");

        let test_data = b"Hello, MIME Type Steganography"; //30 bytes
        let encoded = encoder.encode(&test_data[..]).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(test_data[..], decoded);
        println!("test8 ok");
    }

    fn test_encode_decode_large_for_cfg(cfg: CFG) {
        let encoder = CFGEncoder { cfg };
        // Test case 4.2: REALLY Large data
        let large_data: Vec<u8> = (0..255).collect();
        let repeated: Vec<_> = std::iter::repeat_with(|| large_data.clone())
            .take(1024)
            .flatten()
            .collect();

        let encoded = encoder.encode(&repeated).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(repeated, decoded);

        println!("test4.2 ok");
    }

    #[test]
    fn test_encoder_capacity() {
        let cfg = CFG::example1();
        let encoder = CFGEncoder { cfg };

        // Calculate theoretical capacity
        let mut total_bits = 0;
        for productions in encoder.cfg.production.values() {
            let num_productions = productions.len();
            if num_productions > 1 {
                let bits = (num_productions as f64).log2().floor() as u32;
                total_bits += bits;
            }
        }

        // Test encoding with data size equal to capacity
        let capacity_bytes = total_bits as usize / 8;
        let test_data: Vec<u8> = (0..capacity_bytes).map(|i| i as u8).collect();
        let encoded = encoder.encode(&test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        // The decoded data might be shorter than the original due to padding
        // but all complete bytes should match
        assert_eq!(&test_data[..decoded.len()], &decoded[..]);
    }

    #[test]
    fn test_encode_decode_utf8() {
        let cfg = CFG::example1();
        let encoder = CFGEncoder { cfg };

        // Test with data that contains UTF-8 characters when encoded
        let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
        let encoded = encoder.encode(&data).unwrap();

        // Verify the encoded text is valid UTF-8
        assert!(String::from_utf8(encoded.clone()).is_ok());

        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_invalid_decode() {
        let cfg = CFG::example1();
        let encoder = CFGEncoder { cfg };

        // Test case 1: Invalid UTF-8 sequence
        let invalid_utf8 = vec![0xFF, 0xFF, 0xFF];
        assert!(encoder.decode(&invalid_utf8).is_err());

        // Test case 2: Valid UTF-8 but invalid grammar
        let invalid_grammar = "This is not a valid CFG text".as_bytes().to_vec();
        assert!(encoder.decode(&invalid_grammar).is_err());
    }
}

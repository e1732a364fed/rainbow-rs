use super::*;
use crate::stego::Encoder;
use crate::Result;

pub fn create_news_grammar() -> Grammar {
    let mut grammar = Grammar::new("ARTICLE");

    // 文章结构
    grammar.add_rule(
        "ARTICLE",
        vec![
            Symbol::NonTerminal("SUBJECT_VERB_OBJECT".to_string()),
            Symbol::NonTerminal("DATELINE".to_string()),
            Symbol::NonTerminal("CONTENT".to_string()),
            Symbol::NonTerminal("QUOTE_INTRO".to_string()),
            Symbol::NonTerminal("QUOTE".to_string()),
        ],
        1,
    );

    // 主语-谓语-宾语结构
    grammar.add_rule(
        "SUBJECT_VERB_OBJECT",
        vec![
            Symbol::NonTerminal("OBJECT".to_string()),
            Symbol::NonTerminal("VERB".to_string()),
            Symbol::NonTerminal("SUBJECT".to_string()),
        ],
        1,
    );

    // 主语
    let subjects = vec![
        "Tech giant",
        "Local authorities",
        // "Scientists",
        // "Researchers",
        // "Industry experts",
        // "Market analysts",
        // "Government officials",
        // "Medical professionals",
        // "Environmental activists",
        // "Security researchers",
        // "Financial experts",
        // "Education leaders",
        // "Technology pioneers",
        // "Healthcare providers",
        // "Climate scientists",
        // "Policy makers",
        // "Business leaders",
        // "Innovation experts",
        // "Data scientists",
        // "AI researchers",
        // "Cybersecurity experts",
        // "Space scientists",
        // "Marine biologists",
        // "Energy researchers",
        // "Quantum physicists",
        // "Neuroscientists",
        // "Biotechnology firms",
        // "Software developers",
        // "Automotive engineers",
        // "Aerospace experts",
        // "Economic analysts",
        // "Urban planners",
        // "Agricultural scientists",
        // "Digital strategists",
        // "Robotics engineers",
        // "Chemical researchers",
        // "Investment analysts",
        // "Public health experts",
        // "Technology consultants",
        // "Social scientists",
        // "Military strategists",
        // "Transportation experts",
        // "Renewable energy experts",
        // "Blockchain developers",
    ];
    for subject in subjects {
        grammar.add_rule("SUBJECT", vec![Symbol::Terminal(subject.to_string())], 1);
    }

    // 谓语
    let verbs = vec![
        "reveals",
        "launches",
        // "discovers",
        // "introduces",
        // "develops",
        // "implements",
        // "demonstrates",
        // "unveils",
        // "presents",
        // "confirms",
        // "establishes",
        // "initiates",
        // "validates",
        // "showcases",
        // "releases",
        // "publishes",
        // "verifies",
        // "deploys",
        // "pioneers",
        // "achieves",
        // "creates",
        // "designs",
        // "patents",
        // "revolutionizes",
        // "transforms",
        // "advances",
        // "accelerates",
        // "enhances",
        // "optimizes",
        // "modernizes",
        // "reinvents",
    ];
    for verb in verbs {
        grammar.add_rule("VERB", vec![Symbol::Terminal(verb.to_string())], 1);
    }

    // 宾语
    let objects = vec![
        "new findings",
        "innovative solution",
        // "major development",
        // "groundbreaking research",
        // "revolutionary platform",
        // "cutting-edge system",
        // "advanced framework",
        // "sustainable initiative",
        // "strategic partnership",
        // "quantum breakthrough",
        // "AI-powered solution",
        // "digital transformation",
        // "research findings",
        // "technological advancement",
        // "innovative approach",
        // "sustainable solution",
        // "security protocol",
        // "efficiency improvement",
        // "market strategy",
        // "development framework",
        // "research methodology",
        // "optimization technique",
        // "implementation strategy",
        // "analytical tool",
        // "prediction model",
        // "automation system",
        // "integration platform",
        // "monitoring system",
        // "validation process",
        // "enhancement protocol",
        // "deployment strategy",
        // "scaling solution",
        // "protection mechanism",
        // "acceleration framework",
        // "optimization algorithm",
        // "verification system",
        // "compliance framework",
        // "management platform",
        // "analysis methodology",
    ];
    for object in objects {
        grammar.add_rule("OBJECT", vec![Symbol::Terminal(object.to_string())], 1);
    }

    // 引用部分
    grammar.add_rule(
        "QUOTE",
        vec![Symbol::Terminal(
            "\"This is just the beginning of a new era in technology.\"".to_string(),
        )],
        1,
    );

    // 引用介绍
    grammar.add_rule(
        "QUOTE_INTRO",
        vec![Symbol::Terminal("The lead scientist stated,".to_string())],
        1,
    );

    let contents = vec![
        "This breakthrough could revolutionize the industry. ",
        "The development marks a significant milestone in the field1. ",
        // "The development marks a significant milestone in the field2. ",
        // "The development marks a significant milestone in the field3. ",
        // "a groundbreaking discovery in artificial intelligence announced today Tech giant4",
    ];
    for content in contents {
        grammar.add_rule("CONTENT", vec![Symbol::Terminal(content.to_string())], 1);
    }

    // 日期行
    grammar.add_rule(
        "DATELINE",
        vec![
            Symbol::NonTerminal("DATE".to_string()),
            Symbol::NonTerminal("CITY".to_string()),
        ],
        1,
    );

    // 城市
    let cities = vec![
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
    for city in cities {
        grammar.add_rule("CITY", vec![Symbol::Terminal(city.to_string())], 1);
    }

    // 日期
    // let dates: Vec<String> = {
    //     let months = [
    //         ("January", 31),
    //         ("February", 29), // 2024 is a leap year
    //         ("March", 31),
    //         ("April", 30),
    //         ("May", 31),
    //         ("June", 30),
    //         ("July", 31),
    //         ("August", 31),
    //         ("September", 30),
    //         ("October", 31),
    //         ("November", 30),
    //         ("December", 31),
    //     ];

    //     let mut all_dates = Vec::with_capacity(366);
    //     for (month, days) in months.iter() {
    //         for day in 1..=*days {
    //             all_dates.push(format!("{} {}, 2024", month, day));
    //         }
    //     }
    //     all_dates
    // };
    // for date in dates {
    //     grammar.add_rule("DATE", vec![Symbol::Terminal(date)], 1);
    // }

    grammar
}

#[derive(Debug, Clone)]
pub struct NewsGrammarEncoder {
    grammar: Grammar,
}

impl NewsGrammarEncoder {
    pub fn new() -> Self {
        Self {
            grammar: create_news_grammar(),
        }
    }
}

impl Default for NewsGrammarEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for NewsGrammarEncoder {
    fn name(&self) -> &'static str {
        "news_grammar"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        let (text, _) = self.grammar.generate(data);
        Ok(text.into_bytes())
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        let text = String::from_utf8_lossy(content);
        let decoded = self.grammar.decode(&text);
        Ok(decoded)
    }

    fn get_mime_type(&self) -> &'static str {
        "text/plain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_grammar_encoder_basic() {
        let encoder = NewsGrammarEncoder::new();

        // 测试数据（一个字节）
        let test_data = vec![0xA5]; // 10100101
        println!("\nInput byte: {:08b}", test_data[0]);

        // 编码数据
        let encoded = encoder.encode(&test_data).unwrap();

        // 验证编码的数据是有效的 UTF-8
        let encoded_text = String::from_utf8(encoded.clone()).unwrap();
        println!("Encoded text: {}", encoded_text);

        // 解码数据
        let decoded = encoder.decode(&encoded).unwrap();
        println!("Decoded byte: {:08b}", decoded[0]);

        assert_eq!(decoded, test_data, "Decoded data should match input data");
        assert!(!decoded.is_empty(), "Decoded data should not be empty");
    }

    #[test]
    fn test_news_grammar_structure() {
        let encoder = NewsGrammarEncoder::new();

        // 生成一些示例文本（使用一个字节）
        let test_data = vec![0x55]; // 01010101
        let encoded = encoder.encode(&test_data).unwrap();
        let text = String::from_utf8(encoded).unwrap();
        println!("Generated text: {}", text);

        // 验证文章结构的完整性
        assert!(
            text.contains("Tech giant")
                || text.contains("Local authorities")
                || text.contains("Scientists")
                || text.contains("Researchers"),
            "Should contain a valid subject"
        );

        assert!(
            text.contains("reveals")
                || text.contains("launches")
                || text.contains("discovers")
                || text.contains("introduces"),
            "Should contain a valid verb"
        );

        assert!(
            text.contains("NEW YORK")
                || text.contains("LONDON")
                || text.contains("TOKYO")
                || text.contains("BEIJING"),
            "Should contain a valid city"
        );

        // 更新日期检查逻辑，使用正则表达式匹配任意有效日期
        let date_pattern = r"(January|February|March|April|May|June|July|August|September|October|November|December) \d{1,2}, 2024";
        let re = regex::Regex::new(date_pattern).unwrap();
        assert!(
            re.is_match(&text),
            "Should contain a valid date matching pattern: {}",
            date_pattern
        );

        // 验证引用部分
        assert!(
            text.contains("According to the researchers")
                || text.contains("The lead scientist stated"),
            "Should contain a quote introduction"
        );
    }

    #[test]
    fn test_empty_input() {
        let encoder = NewsGrammarEncoder::new();

        // Test encoding empty data
        let encoded = encoder.encode(&[]).unwrap();
        assert!(
            encoded.is_empty(),
            "Should not still generate some valid text"
        );

        // Verify the encoded text is valid UTF-8
        let text = String::from_utf8(encoded.clone()).unwrap();
        println!("Empty input generated: {}", text);
    }

    #[test]
    fn test_roundtrip_encoding() {
        let encoder = NewsGrammarEncoder::new();
        let grammar = create_news_grammar();

        // 打印语法的容量信息
        let (total_capacity, capacities) = grammar.calculate_capacity();
        println!("\nGrammar capacity: {} bits", total_capacity);
        println!("Capacity by non-terminal:");
        for (nt, bits) in capacities.iter() {
            println!("  {}: {} bits", nt, bits);
        }

        // 使用各种字节模式进行测试
        let test_cases = vec![
            // vec![0x00], // 00000000
            // vec![0xFF], // 11111111
            // vec![0xA5], // 10100101
            // vec![0x5A],             // 01011010
            vec![0xA5, 0x5A], // 两个字节
                              // vec![0x12, 0x34, 0x56], // 三个字节
        ];

        for input in test_cases {
            println!("\nTesting input: {:?}", input);

            // 编码
            let encoded = encoder.encode(&input).unwrap();
            let text = String::from_utf8(encoded.clone()).unwrap();
            println!("Generated text:\n{}", text);

            // 解码
            let decoded = encoder.decode(&encoded).unwrap();
            println!("Decoded bytes: {:?}", decoded);

            println!("encoded: {:?}", String::from_utf8(encoded).unwrap());

            println!(
                "Decoded length should match input length: {:?} vs {:?}",
                decoded, input
            );
            // 验证解码结果与输入匹配
            assert_eq!(
                decoded.len(),
                input.len(),
                "Decoded length should match input length"
            );
            assert_eq!(decoded, input, "Decoded bytes should match input bytes");

            // 验证文本包含必要的新闻元素
            assert!(
                text.contains("Tech giant")
                    || text.contains("Local authorities")
                    || text.contains("Scientists")
                    || text.contains("Researchers"),
                "Should contain a valid subject"
            );

            assert!(
                text.contains("reveals")
                    || text.contains("launches")
                    || text.contains("discovers")
                    || text.contains("introduces"),
                "Should contain a valid verb"
            );
        }
    }

    #[test]
    fn test_news_grammar_capacity() {
        let grammar = create_news_grammar();
        let (total_capacity, capacities) = grammar.calculate_capacity();

        println!("\n=== News Grammar Capacity Analysis ===");
        println!("Total capacity: {} bits", total_capacity);
        println!("\nCapacity breakdown by non-terminal:");

        // 按容量大小排序
        let mut sorted_capacities: Vec<_> = capacities.iter().collect();
        sorted_capacities.sort_by(|a, b| b.1.cmp(a.1));

        for (nt, bits) in sorted_capacities {
            // 计算该非终结符的选项数和在语法中的重复次数
            let options = if let Some(productions) = grammar.rules.get(nt) {
                productions.len()
            } else {
                0
            };

            // 计算该非终结符在各个产生式中的总出现次数
            let mut total_occurrences = 0;
            for (_, productions) in &grammar.rules {
                for prod in productions {
                    for symbol in &prod.symbols {
                        if let Symbol::NonTerminal(ref name) = symbol {
                            if name == nt {
                                total_occurrences += 1;
                            }
                        }
                    }
                }
            }

            println!(
                "{:<15} : {:>2} bits ({:>2} options, used {:>2} times)",
                nt, bits, options, total_occurrences
            );
        }

        // 分析特定结构的容量
        println!("\nDetailed capacity analysis:");

        // BODY 结构分析
        if let Some(body_cap) = capacities.get("BODY") {
            println!("\nBODY structure:");
            println!("  BODY capacity: {} bits", body_cap);
            if let Some(para_cap) = capacities.get("PARAGRAPH") {
                println!(
                    "  PARAGRAPH capacity: {} bits (used twice in BODY)",
                    para_cap
                );
                if let Some(sent_cap) = capacities.get("SENTENCE") {
                    println!(
                        "  SENTENCE capacity: {} bits (used twice in each PARAGRAPH)",
                        sent_cap
                    );
                    println!(
                        "  Total BODY theoretical capacity: {} bits",
                        sent_cap * 4 // 2 sentences × 2 paragraphs
                    );
                }
            }
        }

        // 验证关键非终结符的容量
        assert!(
            capacities.get("SUBJECT").unwrap() >= &2,
            "SUBJECT should encode at least 2 bits (4 options)"
        );
        assert!(
            capacities.get("VERB").unwrap() >= &2,
            "VERB should encode at least 2 bits (4 options)"
        );
        assert!(
            capacities.get("OBJECT").unwrap() >= &2,
            "OBJECT should encode at least 2 bits (4 options)"
        );
        assert!(
            capacities.get("CITY").unwrap() >= &2,
            "CITY should encode at least 2 bits (4 options)"
        );
        assert!(
            capacities.get("QUOTE").unwrap() >= &2,
            "QUOTE should encode at least 2 bits (4 options)"
        );

        // 验证总容量考虑了重复使用
        let sentence_capacity = *capacities.get("SENTENCE").unwrap();
        let expected_min_body_capacity = sentence_capacity * 4; // 2 paragraphs × 2 sentences
        assert!(
            total_capacity >= expected_min_body_capacity,
            "Total capacity should account for repeated SENTENCE usage in BODY"
        );
    }

    #[test]
    fn test_bits_consistency() {
        let encoder = NewsGrammarEncoder::new();
        let input = vec![1];

        let encoded = encoder.encode(&input).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(
            input.len(),
            decoded.len(),
            "Decoded bits length should match input length"
        );
        assert_eq!(input, decoded, "Decoded bits should match input bits");
    }
}

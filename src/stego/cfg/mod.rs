use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

pub mod news;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    rules: HashMap<String, Vec<Production>>,
    start_symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Production {
    symbols: Vec<Symbol>,
    weight: u32, // 用于控制生成概率
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
}

impl Production {
    fn iter(&self) -> std::slice::Iter<'_, Symbol> {
        self.symbols.iter()
    }
}

impl Grammar {
    pub fn new(start_symbol: &str) -> Self {
        Self {
            rules: HashMap::new(),
            start_symbol: start_symbol.to_string(),
        }
    }

    pub fn add_rule(&mut self, non_terminal: &str, production: Vec<Symbol>, weight: u32) {
        self.rules
            .entry(non_terminal.to_string())
            .or_insert_with(Vec::new)
            .push(Production {
                symbols: production,
                weight,
            });
    }

    fn bits_needed(productions_len: usize) -> usize {
        if productions_len <= 1 {
            println!("productions_len <= 1");
            0
        } else {
            println!(
                " (productions_len as f64).log2(): {}",
                (productions_len as f64).log2()
            );
            (productions_len as f64).log2().ceil() as usize
        }
    }

    pub fn generate(&self, bytes: &[u8]) -> (String, usize) {
        let bits = bytes_to_bits(bytes);
        let mut output = String::new();
        let mut bits_used = 0;
        let mut word_positions = Vec::new();
        let mut next_position = 0;

        let mut stack = vec![self.start_symbol.as_str()];

        println!("start_symbol: {}", self.start_symbol);
        let mut visited = HashSet::new();

        // 只要还有未使用的比特，就继续生成
        while bits_used < bits.len() {
            println!("bits_used: {}", bits_used);
            stack.push(self.start_symbol.as_str());
            visited.clear();

            while let Some(non_terminal) = stack.pop() {
                println!("non_terminal: {}", non_terminal);
                // 避免重复访问相同的非终结符和位置组合
                if !visited.insert(non_terminal.to_string()) {
                    continue;
                }

                if let Some(productions) = self.rules.get(non_terminal) {
                    println!("productions: {:?}", productions);
                    let bits_needed = Self::bits_needed(productions.len());

                    println!("bits_needed: {}", bits_needed);

                    println!("bits_used: {}", bits_used);
                    println!("bits.len(): {}", bits.len());

                    // 如果没有足够的比特，使用第一个产生式
                    if bits_used >= bits.len() {
                        let production = &productions[0];
                        for symbol in production.symbols.iter() {
                            match symbol {
                                Symbol::Terminal(s) => {
                                    word_positions.push((next_position, s.as_str()));
                                    next_position += 1;
                                }
                                Symbol::NonTerminal(nt) => {
                                    stack.push(nt.as_str());
                                }
                            }
                        }
                        continue;
                    }

                    // 计算可用的比特数
                    let available_bits = bits_needed.min(bits.len() - bits_used);
                    let mut index = 0usize;

                    // 从比特流中读取索引
                    for i in (0..available_bits).rev() {
                        index = (index << 1) | ((bits[bits_used + i] & 1) as usize);
                    }

                    // 确保索引在有效范围内
                    let original_index = index;
                    index = index % productions.len();

                    // 打印调试信息（确保不会越界）
                    if bits_needed > 0 {
                        let debug_start = bits_used;
                        let debug_end = (bits_used + bits_needed).min(bits.len());
                        let debug_bits = if debug_start < debug_end {
                            &bits[debug_start..debug_end]
                        } else {
                            &[]
                        };
                        println!(
                            "Choosing production {} from {} options using {} bits: {:?} (original index: {}, bits_used: {})",
                            index,
                            productions.len(),
                            bits_needed,
                            debug_bits,
                            original_index,
                            bits_used
                        );
                    }

                    bits_used += bits_needed;

                    let production = &productions[index];
                    for symbol in production.symbols.iter() {
                        match symbol {
                            Symbol::Terminal(s) => {
                                word_positions.push((next_position, s.as_str()));
                                next_position += 1;
                                println!(
                                    " word_positions.push((next_position, s.as_str()));, next_position: {}, s.as_str(): {}",
                                    next_position, s.as_str()
                                );
                            }
                            Symbol::NonTerminal(nt) => {
                                println!(" stack.push(nt.as_str()); {}", nt.as_str());
                                stack.push(nt.as_str());
                            }
                        }
                    }
                }
            }
        }

        word_positions.sort_by_key(|(pos, _)| *pos);
        for (i, (_, word)) in word_positions.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            output.push_str(word);
        }

        // 返回实际使用的字节数（向上取整）
        let bytes_used = (bits_used) / 8;
        (output, bytes_used)
    }

    fn choose_production<'a>(
        &self,
        productions: &'a [Production],
        bits: &[u8],
    ) -> (&'a Production, usize) {
        let bits_needed = Self::bits_needed(productions.len());
        if bits_needed == 0 || bits.is_empty() {
            return (&productions[0], 0);
        }

        let mut index = 0;
        let available_bits = bits_needed.min(bits.len());

        for i in 0..available_bits {
            if bits[i] == 1 {
                index |= 1 << i;
            }
        }

        index = index % productions.len();

        println!(
            "Choosing production {} from {} options using {} bits: {:?}",
            index,
            productions.len(),
            available_bits,
            &bits[..available_bits]
        );

        (&productions[index], bits_needed)
    }

    pub fn decode(&self, text: &str) -> Vec<u8> {
        let mut decoded_bits = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut pos = 0;

        // 从起始符号开始解码
        let mut stack = vec![self.start_symbol.as_str()];
        let mut visited = HashSet::new();

        while let Some(non_terminal) = stack.pop() {
            // 避免重复访问相同的非终结符和位置组合
            if !visited.insert(non_terminal.to_string()) {
                continue;
            }

            if let Some(productions) = self.rules.get(non_terminal) {
                let bits_needed = Self::bits_needed(productions.len());
                let mut matched = false;

                // 尝试每个产生式
                'production_loop: for (i, production) in productions.iter().enumerate() {
                    let mut current_pos = pos;
                    let mut temp_stack = Vec::new();

                    // 检查每个符号是否匹配
                    for symbol in &production.symbols {
                        match symbol {
                            Symbol::Terminal(expected) => {
                                if current_pos >= words.len() {
                                    continue 'production_loop;
                                }
                                let expected_words: Vec<&str> =
                                    expected.split_whitespace().collect();
                                if current_pos + expected_words.len() > words.len() {
                                    continue 'production_loop;
                                }
                                let actual_words =
                                    &words[current_pos..current_pos + expected_words.len()];
                                let actual = actual_words.join(" ");
                                println!(
                                    "Comparing at pos {}: expected '{}' vs actual '{}'",
                                    current_pos, expected, actual
                                );
                                if actual != *expected {
                                    continue 'production_loop;
                                }
                                current_pos += expected_words.len();
                            }
                            Symbol::NonTerminal(nt) => {
                                temp_stack.push(nt.as_str());
                            }
                        }
                    }

                    // 匹配成功
                    if bits_needed > 0 {
                        // 将索引转换为比特序列
                        let mut index_bits = Vec::with_capacity(bits_needed);
                        let mut index = i;
                        for i in 0..bits_needed {
                            index_bits.push(((index >> i) & 1) as u8);
                        }
                        println!(
                            "Decode: matched production {} of {} at position {}, adding {} bits: {:?}",
                            i,
                            productions.len(),
                            pos,
                            bits_needed,
                            index_bits
                        );
                        decoded_bits.extend(index_bits);
                    }

                    pos = current_pos;
                    stack.extend(temp_stack);
                    matched = true;
                    break;
                }

                if !matched {
                    println!(
                        "Failed to match any production for non-terminal: {} at position {}",
                        non_terminal, pos
                    );
                    println!("Current words: {:?}", &words[pos..]);
                    println!("Available productions:");
                    for (i, prod) in productions.iter().enumerate() {
                        println!("  {}: {:?}", i, prod.symbols);
                    }
                    return Vec::new();
                }
            }
        }

        // 计算需要的完整字节数
        let actual_bits_used = decoded_bits.len();
        let bytes_needed = (actual_bits_used) / 8;

        // 确保 decoded_bits 的长度是 8 的倍数
        while decoded_bits.len() < bytes_needed * 8 {
            decoded_bits.push(0);
        }
        decoded_bits.truncate(bytes_needed * 8);

        println!("Final decoded bits: {:?}", decoded_bits);

        // 将比特转换为字节
        let mut result = Vec::with_capacity(1);
        let mut byte = 0u8;
        let mut pos = 0;

        for &bit in decoded_bits.iter().take(8) {
            byte = (byte << 1) | (bit & 1);
            pos += 1;
        }

        result.push(byte);
        result
    }

    fn bits_needed_for_non_terminal(&self, non_terminal: &str) -> usize {
        if let Some(productions) = self.rules.get(non_terminal) {
            Self::bits_needed(productions.len())
        } else {
            0
        }
    }

    /// 计算语法的最大隐写容量（以比特为单位）
    ///
    /// 对于每个非终结符：
    /// - 如果有 N 个产生式选项，每次使用可以编码 log2(N) 向下取整的比特
    /// - 如果一个非终结符在一个产生式中出现多次，其容量会相应倍增
    ///
    /// 返回：
    /// - 总容量：语法能编码的最大比特数
    /// - 每个非终结符的单次使用容量映射：用于调试和优化
    pub fn calculate_capacity(&self) -> (usize, HashMap<String, usize>) {
        let mut memo = HashMap::new();
        let mut capacities = HashMap::new();

        // 从起始符号开始计算
        let (bits, visited) = self.calculate_capacity_recursive(&self.start_symbol, &mut memo);
        let total_bits = bits;

        // 收集所有访问过的非终结符的容量（单次使用）
        for (nt, bits) in visited {
            capacities.insert(nt, bits);
        }

        (total_bits, capacities)
    }

    /// 递归计算非终结符的容量，考虑重复使用
    fn calculate_capacity_recursive(
        &self,
        non_terminal: &str,
        memo: &mut HashMap<String, (usize, HashSet<String>)>,
    ) -> (usize, HashMap<String, usize>) {
        // 如果已经计算过，直接返回缓存的结果
        if let Some((bits, visited)) = memo.get(non_terminal) {
            let mut result = HashMap::new();
            for nt in visited {
                if let Some((cached_bits, _)) = memo.get(nt) {
                    result.insert(nt.clone(), *cached_bits);
                }
            }
            return (*bits, result);
        }

        let mut total_bits = 0;
        let mut visited = HashSet::new();
        visited.insert(non_terminal.to_string());
        let mut sub_capacities = HashMap::new();

        if let Some(productions) = self.rules.get(non_terminal) {
            // 计算当前非终结符的选择容量
            let choice_bits = if productions.len() <= 1 {
                0
            } else {
                (productions.len() as f64).log2().floor() as usize
            };

            // 找到容量最大的产生式路径
            let mut max_production_bits = 0;
            for prod in productions {
                let mut prod_bits = 0;
                let mut symbol_counts = HashMap::new();

                // 统计每个非终结符在这个产生式中出现的次数
                for symbol in &prod.symbols {
                    if let Symbol::NonTerminal(nt) = symbol {
                        *symbol_counts.entry(nt.clone()).or_insert(0) += 1;
                    }
                }

                // 计算这个产生式的总容量
                for (nt, count) in symbol_counts {
                    if !visited.contains(&nt) {
                        let (sub_bits, sub_visited) = self.calculate_capacity_recursive(&nt, memo);
                        visited.extend(sub_visited.keys().cloned());
                        sub_capacities.extend(sub_visited);
                        // 考虑重复使用带来的容量倍增
                        prod_bits += sub_bits * count;
                    }
                }

                max_production_bits = max_production_bits.max(prod_bits);
            }

            total_bits = choice_bits + max_production_bits;
        }

        // 缓存结果
        memo.insert(non_terminal.to_string(), (total_bits, visited.clone()));

        // 添加当前非终结符的容量
        sub_capacities.insert(non_terminal.to_string(), total_bits);

        (total_bits, sub_capacities)
    }
}

// 辅助函数：将索引转换为比特序列
fn index_to_bits(index: usize, bits_needed: usize) -> Vec<u8> {
    let mut bits = vec![0; bits_needed];
    for i in 0..bits_needed {
        bits[i] = ((index >> (bits_needed - 1 - i)) & 1) as u8;
    }
    println!(
        "index_to_bits: index={}, bits_needed={}, bits={:?}",
        index, bits_needed, bits
    );
    bits
}

fn bits_to_index(bits: &[u8], start: usize, max: usize) -> usize {
    let mut index = 0;
    let bits_needed = (max - 1).count_ones() as usize;

    for i in 0..bits_needed {
        if start + i < bits.len() {
            index |= ((bits[start + i] & 1) as usize) << i;
        }
    }

    index % max
}

// 辅助函数：将 u8 slice 转换为比特序列
fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for &byte in bytes {
        for i in 0..8 {
            bits.push((byte >> i) & 1);
        }
    }
    println!("bytes_to_bits: bytes={:?}, bits={:?}", bytes, bits);
    bits
}

// 辅助函数：将比特序列转换为 u8 slice
fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((bits.len() + 7) / 8);
    let mut byte = 0u8;
    let mut pos = 0;

    for &bit in bits {
        byte = (byte << 1) | (bit & 1);
        pos += 1;
        if pos == 8 {
            bytes.push(byte);
            byte = 0;
            pos = 0;
        }
    }

    if pos > 0 {
        byte <<= 8 - pos;
        bytes.push(byte);
    }

    println!("bits_to_bytes: bits={:?}, bytes={:?}", bits, bytes);
    bytes
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calculate_capacity() {
        let mut grammar = Grammar::new("S");

        // S -> A B | C D  (1 bit)
        grammar.add_rule(
            "S",
            vec![
                Symbol::NonTerminal("A".to_string()),
                Symbol::NonTerminal("B".to_string()),
            ],
            1,
        );
        grammar.add_rule(
            "S",
            vec![
                Symbol::NonTerminal("C".to_string()),
                Symbol::NonTerminal("D".to_string()),
            ],
            1,
        );

        // A -> a1 | a2 | a3 | a4  (2 bits)
        grammar.add_rule("A", vec![Symbol::Terminal("a1".to_string())], 1);
        grammar.add_rule("A", vec![Symbol::Terminal("a2".to_string())], 1);
        grammar.add_rule("A", vec![Symbol::Terminal("a3".to_string())], 1);
        grammar.add_rule("A", vec![Symbol::Terminal("a4".to_string())], 1);

        // B -> b1 | b2  (1 bit)
        grammar.add_rule("B", vec![Symbol::Terminal("b1".to_string())], 1);
        grammar.add_rule("B", vec![Symbol::Terminal("b2".to_string())], 1);

        // C -> c1 | c2  (1 bit)
        grammar.add_rule("C", vec![Symbol::Terminal("c1".to_string())], 1);
        grammar.add_rule("C", vec![Symbol::Terminal("c2".to_string())], 1);

        // D -> d  (0 bits)
        grammar.add_rule("D", vec![Symbol::Terminal("d".to_string())], 1);

        let (total_capacity, capacities) = grammar.calculate_capacity();

        // 验证总容量
        // S: 1 bit (2 productions)
        // Path 1: A(2 bits) + B(1 bit) = 3 bits
        // Path 2: C(1 bit) + D(0 bits) = 1 bit
        // 总容量应该是最大路径：1 + max(3, 1) = 4 bits
        assert_eq!(total_capacity, 4, "Total capacity should be 4 bits");

        // 验证各非终结符的容量
        assert_eq!(capacities["S"], 4, "S should have capacity of 4 bits");
        assert_eq!(capacities["A"], 2, "A should have capacity of 2 bits");
        assert_eq!(capacities["B"], 1, "B should have capacity of 1 bit");
        assert_eq!(capacities["C"], 1, "C should have capacity of 1 bit");
        assert_eq!(capacities["D"], 0, "D should have capacity of 0 bits");
    }
}

use std::collections::{btree_set, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Item {
    rule_id: (usize, String),
    lhs: String,
    rhs: Vec<String>,
    pointer: usize,
}

impl Item {
    pub fn from(rule_id: (usize, String), lhs: String, rhs: Vec<String>, pointer: usize) -> Self {
        Self {
            rule_id,
            lhs,
            rhs,
            pointer,
        }
    }

    pub fn get_symbol_under_pointer(&self) -> Option<String> {
        self.rhs.get(self.pointer).cloned()
    }

    pub fn inc_pointer(&mut self) {
        self.pointer += 1;
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rule_id.0, self.pointer).cmp(&(other.rule_id.0, other.pointer))
    }
}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (s1, s2) = self.rhs.iter().enumerate().fold(
            ("".to_string(), "".to_string()),
            |(mut s1, mut s2), (i, symbol)| {
                if i < self.pointer {
                    s1.push_str(" ");
                    s1.push_str(&symbol);
                } else {
                    s2.push_str(" ");
                    s2.push_str(&symbol);
                }
                (s1, s2)
            },
        );

        write!(
            f,
            "{} ->{} .{} ({}, {})",
            self.lhs, s1, s2, self.rule_id.0, self.rule_id.1
        )
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LR0Closure(BTreeSet<Item>);
impl LR0Closure {
    pub fn from<T: IntoIterator<Item = Item>>(
        items: T,
        map_lhs2items: &HashMap<String, HashSet<Item>>, // lhs -> 左辺がlhsのitemの集合
    ) -> Self {
        let mut item_set = BTreeSet::from_iter(items);

        let mut vdq: VecDeque<_> = item_set.clone().into_iter().collect();

        while let Some(item) = vdq.pop_front() {
            // もし次に読む文字が非終端記号なら，その文字がlhsであるようなitemを追加
            if let Some(x) = item.get_symbol_under_pointer() {
                if let Some(set) = map_lhs2items.get(&x) {
                    for wrp_with_lhs_x in set {
                        if item_set.insert(wrp_with_lhs_x.clone()) {
                            vdq.push_back(wrp_with_lhs_x.clone());
                        }
                    }
                }
            }
        }

        LR0Closure(item_set)
    }
}

impl<'a> std::iter::IntoIterator for &'a LR0Closure {
    type Item = &'a Item;
    type IntoIter = btree_set::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

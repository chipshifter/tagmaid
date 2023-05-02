use super::tag_util;
use anyhow::{bail, Result};
use std::collections::HashSet;
//Until database is finished
pub trait DatabaseCount {
    fn get_tag_count(&self, name: &str) -> u64;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SearchOption {
    Tag(String),
    TagSet(Vec<SearchOption>),
    NotEmpty,
    Not(Box<SearchOption>),
    OrEmpty,
    Or(Box<SearchOption>),
}
impl SearchOption {
    /// Try to add a character to the inner string
    pub fn add_char(&mut self, c: char) -> Result<()> {
        match self {
            Self::Tag(s) => s.push(c),
            Self::TagSet(_) => {
                bail!("Cannot add char to tagset");
            }
            Self::NotEmpty => {
                *self = Self::Not({
                    let mut s = String::new();
                    s.push(c);
                    Box::new(Self::Tag(s))
                });
            }
            Self::OrEmpty => {
                *self = Self::Or({
                    let mut s = String::new();
                    s.push(c);
                    Box::new(Self::Tag(s))
                });
            }
            Self::Not(t) | Self::Or(t) => {
                return t.add_char(c);
            }
        }
        Ok(())
    }
    /// Check that a tag's characters fit the validations function and make sure that the Search Option isn't empty
    pub fn verify(&self) -> Result<()> {
        match self {
            Self::Tag(t) => {
                tag_util::validate_tag_name(t)?;
            }
            Self::TagSet(v) => {
                for i in v.iter() {
                    i.verify()?;
                }
            }
            Self::NotEmpty => {
                bail!("Empty not");
            }
            Self::OrEmpty => {
                bail!("Empty or")
            }
            Self::Not(t) | Self::Or(t) => {
                t.verify()?;
            }
        }
        Ok(())
    }
    fn or(&self) -> bool {
        match self {
            Self::Or(..) => true,
            _ => false,
        }
    }
    /// Returns true if the inner string 
    fn filter_single(&self, tags: &HashSet<String>) -> bool {
        match self {
            SearchOption::Tag(s) => tags.contains(s),
            SearchOption::TagSet(set) => _filter_post(set, tags),
            SearchOption::Not(t) => !t.filter_single(tags),
            SearchOption::Or(t) => t.filter_single(tags),
            _ => {
                panic!("Invalid search option");
            }
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Search {
    v: Vec<SearchOption>,
}
impl Search {
    fn new(v: Vec<SearchOption>) -> Self {
        Self { v }
    }
    pub fn from_string(s: &str) -> Result<Search> {
        let search = Self::_search_from_string(&mut s.chars(), 0)?;
        for i in search.iter() {
            i.verify()?;
        }
        Ok(Search::new(search))
    }
    pub fn _search_from_string(
        chars: &mut std::str::Chars,
        depth: u8,
    ) -> Result<Vec<SearchOption>> {
        let mut last = ' ';
        let mut fin = Vec::new();
        while let Some(i) = chars.next() {
            match i {
                // Tag set
                '[' => {
                    // If the tagset should go inside some tag modifier (ie. "not" or "or"). -[inner...] -> Not(TagSet[inner...])
                    match fin.last() {
                        Some(SearchOption::NotEmpty) => {
                            *fin.last_mut().unwrap() = SearchOption::Not(Box::new(
                                SearchOption::TagSet(Self::_search_from_string(chars, depth + 1)?),
                            ));
                        }
                        Some(SearchOption::OrEmpty) => {
                            *fin.last_mut().unwrap() = SearchOption::Or(Box::new(
                                SearchOption::TagSet(Self::_search_from_string(chars, depth + 1)?),
                            ));
                        }
                        _ => {
                            fin.push(SearchOption::TagSet(Self::_search_from_string(
                                chars,
                                depth + 1,
                            )?));
                        }
                    }
                }
                // Exit tagset
                ']' => {
                    if depth == 0 {
                        bail!("Unexpected \"]\" ");
                    } else {
                        return Ok(fin);
                    }
                }
                // None or maybe part of a tag
                '-' => {
                    // Tag character
                    if last != '[' && last != ']' && last != '~' && last != ' ' {
                        match fin.last_mut() {
                            None | Some(SearchOption::TagSet(..)) => {
                                fin.push(SearchOption::NotEmpty);
                            }
                            Some(l) => {
                                l.add_char(i)?;
                            }
                        }
                    } else {
                        // Not
                        fin.push(SearchOption::NotEmpty);
                    }
                }
                // Or
                '~' => {
                    fin.push(SearchOption::OrEmpty);
                }
                ' ' => {} // Do nothing
                // Tag character
                ch => {
                    if last != '[' && last != ']' && last != ' ' {
                        match fin.last_mut() {
                            None | Some(SearchOption::TagSet(..)) => {
                                fin.push({
                                    let mut s = String::new();
                                    s.push(ch);
                                    SearchOption::Tag(s)
                                });
                            }
                            Some(l) => {
                                l.add_char(ch)?;
                            }
                        }
                    } else {
                        fin.push({
                            let mut s = String::new();
                            s.push(ch);
                            SearchOption::Tag(s)
                        });
                    }
                }
            }
            last = i;
        }
        Ok(fin)
    }
    /// true if matches query. false if it doesn't
    pub fn filter_post(&self, tags: &HashSet<String>) -> bool {
        _filter_post(&self.v, tags)
    }
    pub fn initial_tag(&self, d: &dyn DatabaseCount) -> Option<String> {
        _initial_search_tag(&self.v, d)
    }
    pub fn first_tag(&self) -> Option<String> {
        _first_tag(&self.v)
    }
}

/// Recursive call back for TagSets
fn _filter_post(s: &Vec<SearchOption>, tags: &HashSet<String>) -> bool {
    let ors: Vec<&SearchOption> = s.iter().filter(|&x| x.or()).collect();
    for i in s.iter() {
        if !i.or() {
            if !i.filter_single(tags) {
                return false;
            }
        }
    }
    if ors.len() > 0 {
        let mut or = false;
        for i in ors.iter() {
            or |= i.filter_single(tags);
        }
        if !or {
            return false;
        }
    }
    true
}

/// Recursive callback for TagSets
fn _first_tag(search: &Vec<SearchOption>) -> Option<String> {
    for i in search.iter() {
        match i {
            SearchOption::Tag(t) => {
                return Some(t.clone());
            }
            SearchOption::TagSet(ts) => {
                return _first_tag(&ts);
            }
            _ => {}
        }
    }
    None
}

/// Recursive callback for TagSets
fn _initial_search_tag(search: &Vec<SearchOption>, d: &dyn DatabaseCount) -> Option<String> {
    let mut pair: Option<(u64, String)> = None;
    for i in search.iter() {
        match i {
            SearchOption::Tag(s) => match &mut pair {
                Some(pair) => {
                    let c = d.get_tag_count(s);
                    if pair.0 > c && c > 0 {
                        pair.1 = s.clone();
                        pair.0 = c;
                    }
                }
                None => {
                    let c = d.get_tag_count(s);
                    if c > 0 {
                        pair = Some((c, s.clone()));
                    }
                }
            },
            SearchOption::TagSet(set) => match &mut pair {
                Some(pair) => {
                    if let Some(tag) = _initial_search_tag(&set, d) {
                        let c = d.get_tag_count(&tag);
                        if pair.0 > c && c > 0 {
                            *pair = (c, tag);
                        }
                    }
                }
                None => {
                    if let Some(tag) = _initial_search_tag(search, d) {
                        let c = d.get_tag_count(&tag);
                        if c > 0 {
                            pair = Some((c, tag));
                        }
                    }
                }
            },
            SearchOption::Not(..) => {}
            SearchOption::Or(..) => {}
            _ => {
                panic!("Invalid search option")
            }
        }
    }
    return pair.map(|x| x.1);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    struct TestDb {
        hm: HashMap<String, u64>,
    }
    impl TestDb {
        fn new(v: Vec<(&str, u64)>) -> Self {
            let mut hm = HashMap::new();
            for i in v.iter() {
                hm.insert(i.0.to_owned(), i.1);
            }
            Self { hm }
        }
    }
    impl DatabaseCount for TestDb {
        fn get_tag_count(&self, name: &str) -> u64 {
            *self.hm.get(name).unwrap_or(&0)
        }
    }

    #[test]
    fn test_parse() {
        let l = Search::from_string("wawa ~wiwi ~wa -[wiwa wauo] lol").unwrap();
        println!("{:?}", l.v);
        assert_eq!(
            l.v,
            vec![
                SearchOption::Tag(String::from("wawa")),
                SearchOption::Or(Box::new(SearchOption::Tag(String::from("wiwi")))),
                SearchOption::Or(Box::new(SearchOption::Tag(String::from("wa")))),
                SearchOption::Not(Box::new(SearchOption::TagSet(vec![
                    SearchOption::Tag(String::from("wiwa")),
                    SearchOption::Tag(String::from("wauo"))
                ]))),
                SearchOption::Tag(String::from("lol"))
            ]
        );
    }
    #[test]
    fn test_filter() {
        let search = Search::from_string("wa -iwi ~ooo ~aaa").unwrap();
        let post_tags: Vec<HashSet<String>> = vec![
            "wa lala ooo".split(" ").map(|x| x.to_owned()).collect(),
            "wa lala iwi ooo".split(" ").map(|x| x.to_owned()).collect(),
            "wa lala".split(" ").map(|x| x.to_owned()).collect(),
            "wa aaa".split(" ").map(|x| x.to_owned()).collect(),
            "wa ooo aaa".split(" ").map(|x| x.to_owned()).collect(),
            "wa lala ooo iwi".split(" ").map(|x| x.to_owned()).collect(),
        ];
        let fin_tags: Vec<bool> = post_tags.iter().map(|x| search.filter_post(x)).collect();
        assert_eq!(fin_tags, vec![true, false, false, true, true, false]);
    }
    #[test]
    fn test_initial_search() {
        let search1 = Search::from_string("cat black_body tongue blep").unwrap();
        let search2 = Search::from_string("dog yellow_body sitting").unwrap();
        let search3 = Search::from_string("bird music_note white_body").unwrap();
        let db = TestDb::new(vec![
            ("cat", 56),
            ("black_body", 73),
            ("tongue", 31),
            ("blep", 5),
            ("dog", 52),
            ("yellow_body", 20),
            ("sitting", 15),
            ("bird", 48),
            ("music_note", 1),
            ("white_body", 63),
        ]);
        assert_eq!(search1.initial_tag(&db), Some(String::from("blep")));
        assert_eq!(search2.initial_tag(&db), Some(String::from("sitting")));
        assert_eq!(search3.initial_tag(&db), Some(String::from("music_note")))
    }
}

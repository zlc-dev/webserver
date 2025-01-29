use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub(crate) enum PlaceHolderType {
    Path,
    Dir,
    Value,
    All,
}

pub(crate) struct PlaceHolder {
    pub(crate) pos: usize,
    pub(crate) tp: PlaceHolderType,
}

pub struct Pattern {
    pub(crate) url: String,
    pub(crate) holders: Vec<PlaceHolder>,
}

impl Pattern {

    pub fn from_str(ps: &str) -> Option<Self> {
        enum State {
            Normal,
            InBrace,
            Accept,
            Deny,
        }

        let mut state = State::Normal;
        let ps = ps.as_bytes();
        let mut buf = Vec::<u8>::new();
        let mut holders = Vec::<PlaceHolder>::new();
        let mut i = 0_usize;
        let mut l = 0_usize;
        let mut is_val: bool = false;
        let mut pos_set = HashSet::<usize>::new();

        loop {
            match state {
                State::Normal => {
                    if i == ps.len() {
                        state = State::Accept;
                        continue;
                    }
                    match ps[i] {
                        b'{' => {
                            i += 1;
                            l = i;
                            state = State::InBrace;
                        },
                        b'?' => {
                            is_val = true;
                            buf.push(b'?');
                            i += 1;
                        },
                        c => {
                            buf.push(c);
                            i += 1;
                        }
                    }
                },
                State::InBrace => {
                    if i == ps.len() {
                        state = State::Deny;
                        continue;
                    }
                    match ps[i] {
                        b'}' => {
                            let pos = buf.len();
                            
                            if pos_set.contains(&pos) {
                                state = State::Deny;
                                continue;
                            }
                            pos_set.insert(pos);

                            let tp = match &ps[l..i] {
                                b"path" | b"p" => PlaceHolderType::Path,
                                b"dir" | b"d" => PlaceHolderType::Dir,
                                b"value" | b"v" => PlaceHolderType::Value,
                                b"all" | b"a" => PlaceHolderType::All,
                                _ => {
                                    if is_val {
                                        PlaceHolderType::Value
                                    } else {
                                        PlaceHolderType::Dir
                                    }
                                }
                            };

                            holders.push(PlaceHolder{pos, tp});

                            state = State::Normal;
                            i += 1;
                        },
                        _ => {
                            i += 1;
                        }
                    }
                },
                State::Accept => {
                    break;
                },
                State::Deny => {
                    return None;
                },
            }
        }

        let url;
        unsafe {
            url = String::from_utf8_unchecked(buf);
        }

        let ret = Pattern {
            url,
            holders,
        };  

        Some(ret)
    }

    pub fn match_url<'a>(&self, url: &'a str) -> Option<Vec<&'a str>> {

        #[derive(Debug)]
        enum State {
            Normal,
            Matching,
            Accept,
            Retry,
            Deny,
        }

        let mut ret = vec![&url[0..0]; self.holders.len()];

        let pat = self.url.as_bytes();
        let url = url.as_bytes();
        let mut i = 0_usize;
        let mut j = 0_usize;
        let mut place_nth = 0_usize;
        let mut state = State::Normal;
        let mut l = 0_usize; 
        let mut r = 0_usize;


        loop {
            match state {
                State::Normal => {
                    if !self.holders.is_empty() && place_nth == self.holders.len() {
                        state = State::Deny;
                        continue;
                    }
                    if !self.holders.is_empty() && self.holders[place_nth].pos == i {
                        if l < r {
                            unsafe {
                                ret[place_nth] = std::str::from_utf8_unchecked(&url[l..r]);
                            }
                            place_nth += 1;
                        }
                        l = j;
                        r = j;
                        state = State::Matching;
                        continue;
                    }

                    if j == url.len() {
                        if i == pat.len() {
                            state = State::Accept;
                            continue;
                        } else {
                            state = State::Deny;
                            continue;
                        }
                    } else if i == pat.len() {
                        state = State::Retry;
                        continue;
                    }
                    
                    if pat[i] == url[j] {
                        i += 1;
                        j += 1;
                    } else {
                        state = State::Retry;
                        continue;
                    }
                },
                State::Matching => {
                    if j == url.len() {
                        if i == pat.len() {
                            r = j;
                            if l < r {
                                state = State::Accept;
                                continue;
                            } else {
                                state = State::Deny;
                                continue;
                            }
                        } else {
                            state = State::Deny;
                            continue;
                        }
                    }

                    if !self.holders.is_empty() && place_nth == self.holders.len() {
                        state = State::Deny;
                        continue;
                    }
                    if match self.holders[place_nth].tp {
                        PlaceHolderType::Path => {
                            url[j] == b'?'
                        },
                        PlaceHolderType::Dir => {
                            url[j] == b'/' || url[j] == b'?'
                        },
                        PlaceHolderType::Value => {
                            url[j] == b'&'
                        },
                        PlaceHolderType::All => false,
                    } {
                        r = j;
                        if l < r {
                            unsafe {
                                ret[place_nth] = std::str::from_utf8_unchecked(&url[l..r]);
                            }
                            place_nth += 1;
                        }
                        l = r;
                        i += 1;
                        j += 1;
                        state = State::Normal;
                    } else if i != pat.len() && pat[i] == url[j] {
                        r = j;
                        i += 1;
                        j += 1;
                        state = State::Normal;
                    } else {
                        j += 1;
                    }

                },
                State::Accept => {
                    if l < r {
                        if place_nth == self.holders.len() {
                            state = State::Deny;
                            continue;
                        }
                        unsafe {
                            ret[place_nth] = std::str::from_utf8_unchecked(&url[l..r]);
                        }
                    }
                    return Some(ret);
                },
                State::Retry => {
                    if l >= r {
                        state = State::Deny;
                    } else {
                        i = self.holders[place_nth].pos;
                        j = r + 1;
                        state = State::Matching;
                    }
                },
                State::Deny => {
                    return None;
                },
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_dir() {
        let pt = Pattern::from_str("/hello/{}/{d}/{dir}").unwrap();
        assert_eq!(pt.url, "/hello///");
        assert_eq!(pt.holders[0].tp, PlaceHolderType::Dir);
        assert_eq!(pt.holders[1].tp, PlaceHolderType::Dir);
        assert_eq!(pt.holders[2].tp, PlaceHolderType::Dir);
        assert_eq!(pt.match_url("/hello/world/from/rust"), Some(vec!["world", "from", "rust"]));
    }

    #[test]
    fn test_pattern_dir_value() {
        let pt = Pattern::from_str("/hello/{}?val={}").unwrap();
        assert_eq!(pt.url, "/hello/?val=");
        assert_eq!(pt.holders[0].tp, PlaceHolderType::Dir);
        assert_eq!(pt.holders[1].tp, PlaceHolderType::Value);
        assert_eq!(pt.match_url("/hello/world?val=123"), Some(vec!["world", "123"]));
        assert_eq!(pt.match_url("/hello/world/evil?val=123"), None);
    }

    #[test]
    fn test_pattern_dir_path() {
        let pt = Pattern::from_str("/hello/{d}/{path}").unwrap();
        assert_eq!(pt.url, "/hello//");
        assert_eq!(pt.holders[0].tp, PlaceHolderType::Dir);
        assert_eq!(pt.holders[1].tp, PlaceHolderType::Path);
        assert_eq!(pt.match_url("/hello/world/from"), Some(vec!["world", "from"]));
        assert_eq!(pt.match_url("/hello/world/from/evil"), Some(vec!["world", "from/evil"]));
        assert_eq!(pt.match_url("/hello/world/from/evil/rust"), Some(vec!["world", "from/evil/rust"]));
    }

    #[test]
    fn test_pattern_dir_path_values() {
        let pt = Pattern::from_str("/hello/{d}/{path}?val={}&age={}").unwrap();
        assert_eq!(pt.url, "/hello//?val=&age=");
        assert_eq!(pt.holders[0].tp, PlaceHolderType::Dir);
        assert_eq!(pt.holders[1].tp, PlaceHolderType::Path);
        assert_eq!(pt.holders[2].tp, PlaceHolderType::Value);
        assert_eq!(pt.holders[3].tp, PlaceHolderType::Value);
        assert_eq!(pt.match_url("/hello/world/from?val=123&age=24"), Some(vec!["world", "from", "123", "24"]));
        assert_eq!(pt.match_url("/hello/world/from/evil?val=123&age=24"), Some(vec!["world", "from/evil", "123", "24"]));
        assert_eq!(pt.match_url("/hello/world/from/evil/rust?val=123&age=24"), Some(vec!["world", "from/evil/rust", "123", "24"]));
    }

    #[test]
    fn test_pattern_fail() {
        assert!(Pattern::from_str("/hello/{d}/{path}?val={}{}&age={}").is_none());
        assert!(Pattern::from_str("/hello/{d}{}/{path}?val={}&age={}").is_none());
        let pt = Pattern::from_str("/hello").unwrap();
        assert_eq!(pt.match_url("/hello"), Some(vec![]));
        assert_eq!(pt.match_url("/hell0"), None);
    }

}

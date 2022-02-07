use bimap::BiMap;
use scoped_tls::scoped_thread_local;
use std::cell::RefCell;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Symbol {
    index: u64,
}

pub struct StringInterner {
    map: BiMap<&'static str, u64>,
    current_index: u64,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: BiMap::new(),
            current_index: 0,
        }
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(index) = self.map.get_by_left(string) {
            return Symbol { index: *index };
        }

        let boxed_slice: Box<str> = string.into();
        let leaked: &'static str = Box::leak(boxed_slice);

        let symbol = Symbol {
            index: self.current_index,
        };
        self.current_index += 1;
        self.map.insert(leaked, symbol.index);

        symbol
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&'static str> {
        if let Some(string) = self.map.get_by_right(&symbol.index) {
            Some(*string)
        } else {
            None
        }
    }
}

pub struct SessionGlobals {
    pub string_interner: RefCell<StringInterner>,
}

scoped_thread_local!(static SESSION_GLOBALS: SessionGlobals);

impl SessionGlobals {
    pub fn new() -> Self {
        Self {
            string_interner: RefCell::new(StringInterner::new()),
        }
    }

    pub fn create(function: impl Fn()) {
        SESSION_GLOBALS.set(&SessionGlobals::new(), function)
    }

    pub fn with<T>(function: impl Fn(&SessionGlobals) -> T) -> T {
        SESSION_GLOBALS.with(function)
    }

    pub fn with_interner<T>(function: impl FnOnce(&StringInterner) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&sg.string_interner.borrow()))
    }

    pub fn with_interner_mut<T>(function: impl FnOnce(&mut StringInterner) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&mut sg.string_interner.borrow_mut()))
    }
}

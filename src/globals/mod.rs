use crate::solidlang::context::pool::{Pool, PoolRef};
use crate::solidlang::context::ty::Ty;
use bimap::BiMap;
use scoped_tls::scoped_thread_local;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use crate::solidlang::context::template::Template;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Symbol {
    index: u64,
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        SessionGlobals::with_interner(|i| i.get(self).unwrap().fmt(f))
    }
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
    pub ty_pool: RefCell<Pool<Ty>>,
    pub template_pool: RefCell<Pool<Template>>
}

scoped_thread_local!(static SESSION_GLOBALS: SessionGlobals);

impl SessionGlobals {
    pub fn new() -> Self {
        Self {
            string_interner: RefCell::new(StringInterner::new()),
            ty_pool: RefCell::new(Pool::new()),
            template_pool: RefCell::new(Pool::new())
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

    pub fn with_ty_pool<T>(function: impl FnOnce(&Pool<Ty>) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&sg.ty_pool.borrow()))
    }

    pub fn with_ty_pool_mut<T>(function: impl FnOnce(&mut Pool<Ty>) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&mut sg.ty_pool.borrow_mut()))
    }

    pub fn with_template_pool<T>(function: impl FnOnce(&Pool<Template>) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&sg.template_pool.borrow()))
    }

    pub fn with_template_pool_mut<T>(function: impl FnOnce(&mut Pool<Template>) -> T) -> T {
        SESSION_GLOBALS.with(|sg| function(&mut sg.template_pool.borrow_mut()))
    }
}

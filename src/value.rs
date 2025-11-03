#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(Integer),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Extension(Extension),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Integer {
    U64(u64),
    I64(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub type_id: i8,
    pub data: Vec<u8>,
}

impl From<i64> for Integer {
    fn from(v: i64) -> Self {
        Self::I64(v)
    }
}
impl From<u64> for Integer {
    fn from(v: u64) -> Self {
        Self::U64(v)
    }
}

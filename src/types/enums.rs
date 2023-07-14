pub enum LinkType {
    /// 短期的
    #[allow(dead_code)]
    INTERIM,
    /// 长期的
    #[allow(dead_code)]
    PERSIST,
}

impl LinkType {
    pub fn to_value(&self) -> i32 {
        match self {
            LinkType::INTERIM => 1,
            LinkType::PERSIST => 2,
        }
    }
}

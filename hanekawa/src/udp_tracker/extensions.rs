// BEP 41: UDP Tracker Protocol Extensions

#[derive(Debug, Eq, PartialEq)]
pub enum Extension {
    Nop,
    UrlData(String),
    Unknown(u8, String),
}

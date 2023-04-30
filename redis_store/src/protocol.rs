#[derive(Debug, Clone)]
pub enum Protocol {
    Get(String),
    Set(String, String), //key, value
    InvalidRequest,
    Ack,
}
impl Protocol {
    pub fn get(value: String) -> Self {
        return Self::Get(value);
    }
    pub fn set(key: String, value: String) -> Self {
        return Self::Set(key, value);
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Set(key, val) => return format!("SET {}\n{}\n", key, val),
            Self::Get(val) => return format!("GET {}\n", val),
            Self::InvalidRequest => "INVALID_REQUEST\n".to_string(),
            Self::Ack => "ACK\n".to_string(),
        }
    }
    pub fn parse(value: String) -> Result<Self, String> {
        let mut commands = value.split('\n');

        let mut first_line = commands.next().unwrap().split(' ');
        match first_line.next().unwrap() {
            "GET" => return Ok(Self::Get(first_line.next().unwrap().to_string())),
            "SET" => {
                let key = first_line.next().unwrap().to_string();

                let value = commands.next().unwrap().to_string();

                return Ok(Self::Set(key, value));
            }
            "INVALID_REQUEST" => return Ok(Self::InvalidRequest),
            "ACK" => return Ok(Self::Ack),
            _ => Err("Invalid value".to_string()),
        }
    }
}
impl PartialEq for Protocol {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Get(k), Self::Get(k2)) => k == k2,
            (Self::Set(k, v), Self::Set(k2, v2)) => k == k2 && v == v2,
            (Self::InvalidRequest, Self::InvalidRequest) => true,
            (Self::Ack, Self::Ack) => true,
            _ => false,
        }
    }
}

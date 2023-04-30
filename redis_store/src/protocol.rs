#[derive(Debug, Clone)]
pub enum Protocol {
    Get(String, usize),
    Set(String, String, usize),
    InvalidRequest,
    Ack,
}
impl Protocol {
    pub fn get(value: String, ts: usize) -> Self {
        return Self::Get(value, ts);
    }
    pub fn set(key: String, value: String, ts: usize) -> Self {
        return Self::Set(key, value, ts);
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Set(key, val, ts) => return format!("SET {} {}\n{}\n", key, ts, val),
            Self::Get(val, ts) => return format!("GET {} {}\n", val, ts),
            Self::InvalidRequest => "INVALID_REQUEST\n".to_string(),
            Self::Ack => "ACK\n".to_string(),
        }
    }
    pub fn get_ts(&self) -> usize {
        match self {
            Self::Get(_, ts) => ts.clone(),
            Self::Set(_, _, ts) => ts.clone(),
            _ => 0,
        }
    }
    pub fn parse(value: String) -> Result<Self, String> {
        let mut commands = value.split('\n');

        let mut first_line = commands.next().unwrap().split(' ');
        match first_line.next().unwrap() {
            "GET" => {
                let key = first_line.next().unwrap().to_string();
                let ts = match first_line.next() {
                    Some(v) => v.parse::<usize>().unwrap(),
                    _ => 0,
                };
                return Ok(Self::Get(key, ts));
            }
            "SET" => {
                let key = first_line.next().unwrap().to_string();
                let ts = match first_line.next() {
                    Some(v) => v.parse::<usize>().unwrap(),
                    _ => 0,
                };

                let value = commands.next().unwrap().to_string();

                return Ok(Self::Set(key, value, ts));
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
            (Self::Get(k, ts), Self::Get(k2, ts2)) => k == k2 && ts == ts2,
            (Self::Set(k, v, ts), Self::Set(k2, v2, ts2)) => k == k2 && v == v2 && ts == ts2,
            (Self::InvalidRequest, Self::InvalidRequest) => true,
            (Self::Ack, Self::Ack) => true,
            _ => false,
        }
    }
}

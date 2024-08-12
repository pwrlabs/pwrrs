pub mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let st = hex::encode(data);
        s.serialize_str(&st)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(d)?;
        if hex_str.len() > 2 && (&hex_str[..2] == "0x" || &hex_str[..2] == "0X") {
            hex::decode(&hex_str[2..])
                .map_err(|e| serde::de::Error::custom(format!("Expected hex string: {e}")))
        } else {
            hex::decode(hex_str)
                .map_err(|e| serde::de::Error::custom(format!("Expected hex string: {e}")))
        }
    }
}

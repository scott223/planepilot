pub enum DataType {
    Float,
    Boolean,
    Integer,
    Empty,
}

pub struct DataStructure {
    pub name: String,
    pub data_type: DataType,
}

pub struct DataIndex {
    pub index: u8,
    pub data: Vec<DataStructure>,
}
pub fn data_map() -> Vec<DataIndex> {
    vec![
        DataIndex {
            index: 17_u8,
            data: vec![
                DataStructure {
                    name: "pitch".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "roll".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "heading_true".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "heading_magnetic".to_string(),
                    data_type: DataType::Float,
                },
            ],
        },
        DataIndex {
            index: 20_u8,
            data: vec![
                DataStructure {
                    name: "latitude".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "longitude".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "alitude_msl".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "altitude_agl".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "on_runway".to_string(),
                    data_type: DataType::Boolean,
                },
            ],
        },
    ]
}

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
            index: 3_u8,
            data: vec![
                DataStructure {
                    name: "Vind".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "Vind".to_string(),
                    data_type: DataType::Empty,
                },
                DataStructure {
                    name: "Vtrue".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "Vground".to_string(),
                    data_type: DataType::Float,
                },
            ],
        },
        DataIndex {
            index: 11_u8,
            data: vec![
                DataStructure {
                    name: "elevator_actual".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "aileron_actual".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "rudder_actual".to_string(),
                    data_type: DataType::Float,
                },
            ],
        },
        DataIndex {
            index: 25_u8,
            data: vec![
                DataStructure {
                    name: "throttle_1_commanded".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_2_commanded".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_3_commanded".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_4_commanded".to_string(),
                    data_type: DataType::Float,
                },
            ],
        },
        DataIndex {
            index: 26_u8,
            data: vec![
                DataStructure {
                    name: "throttle_1_actual".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_2_actual".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_3_actual".to_string(),
                    data_type: DataType::Float,
                },
                DataStructure {
                    name: "throttle_4_actual".to_string(),
                    data_type: DataType::Float,
                },
            ],
        },
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
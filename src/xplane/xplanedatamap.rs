// create the structure and the actual data map that maps the
// index and value from incoming UDP packets to values for the plane state
pub enum DataType {
    Float,
    Boolean,
    Integer,
    Empty,
}
pub struct DataStructure {
    pub name: String,
    pub data_type: DataType,
    pub transformation: Option<f64>,
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
                    transformation: None,
                },
                DataStructure {
                    name: "Vind".to_string(),
                    data_type: DataType::Empty,
                    transformation: None,
                },
                DataStructure {
                    name: "Vtrue".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "Vground".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 4_u8,
            data: vec![
                DataStructure {
                    name: "Mach".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "not_used".to_string(),
                    data_type: DataType::Empty,
                    transformation: None,
                },
                DataStructure {
                    name: "VVI".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "not_used".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "Gload_normal".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "Gload_axial".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "Gload_side".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 11_u8,
            data: vec![
                DataStructure {
                    name: "elevator_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "aileron_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "rudder_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 8_u8,
            data: vec![
                DataStructure {
                    name: "elevator_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "aileron_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "rudder_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 25_u8,
            data: vec![
                DataStructure {
                    name: "throttle_1_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_2_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_3_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_4_commanded".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 26_u8,
            data: vec![
                DataStructure {
                    name: "throttle_1_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_2_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_3_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "throttle_4_actual".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 17_u8,
            data: vec![
                DataStructure {
                    name: "pitch".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "roll".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "heading_true".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "heading_magnetic".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 16_u8,
            data: vec![
                DataStructure {
                    name: "Q".to_string(),
                    data_type: DataType::Float,
                    transformation: Some(57.2958),
                },
                DataStructure {
                    name: "P".to_string(),
                    data_type: DataType::Float,
                    transformation: Some(57.2958),
                },
                DataStructure {
                    name: "R".to_string(),
                    data_type: DataType::Float,
                    transformation: Some(57.2958),
                },
            ],
        },
        DataIndex {
            index: 18_u8,
            data: vec![
                DataStructure {
                    name: "alpha".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "beta".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "hpath".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "vpath".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
            ],
        },
        DataIndex {
            index: 20_u8,
            data: vec![
                DataStructure {
                    name: "latitude".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "longitude".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "altitude_msl".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "altitude_agl".to_string(),
                    data_type: DataType::Float,
                    transformation: None,
                },
                DataStructure {
                    name: "on_runway".to_string(),
                    data_type: DataType::Boolean,
                    transformation: None,
                },
            ],
        },
    ]
}

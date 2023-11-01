use crate::definitions::SqlDataType;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TypeMode {
    Standard,
    Simple,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsonTypeInfo {
    pub type_name: &'static str,
    pub sql_type: SqlDataType,
    pub non_concise_type: SqlDataType,
    pub searchable: i32,
    pub is_case_sensitive: bool,
    pub fixed_prec_scale: bool,
    pub scale: Option<u16>,
    pub precision: Option<u16>,
    pub octet_length: Option<u16>,
    pub fixed_bytes_length: Option<u16>,
    pub literal_prefix: Option<&'static str>,
    pub literal_suffix: Option<&'static str>,
    pub sql_code: Option<i32>,
    pub is_auto_unique_value: Option<bool>,
    pub is_unsigned: Option<bool>,
    pub num_prec_radix: Option<u16>,
    pub simple_type_info: Option<SimpleTypeInfo>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleTypeInfo {
    pub sql_type: SqlDataType,
    pub non_concise_type: SqlDataType,
    pub precision: Option<u16>,
    pub octet_length: Option<u16>,
    pub fixed_bytes_length: Option<u16>,
}

impl SimpleTypeInfo {
    const fn new(precision: u16, octet_length: u16, fixed_bytes_length: u16) -> Option<Self> {
        Some(Self {
            sql_type: SqlDataType::EXT_W_VARCHAR,
            non_concise_type: SqlDataType::EXT_W_VARCHAR,
            precision: Some(precision),
            octet_length: Some(octet_length),
            fixed_bytes_length: Some(fixed_bytes_length),
        })
    }

    const fn default() -> Option<Self> {
        Some(Self {
            sql_type: SqlDataType::EXT_W_VARCHAR,
            non_concise_type: SqlDataType::EXT_W_VARCHAR,
            precision: None,
            octet_length: None,
            fixed_bytes_length: None,
        })
    }
}

pub const SQL_SEARCHABLE: i32 = 3;
pub const SQL_PRED_BASIC: i32 = 2;
pub const SQL_PRED_NONE: i32 = 0;

impl BsonTypeInfo {
    pub const DOUBLE: BsonTypeInfo = BsonTypeInfo {
        type_name: "double",
        sql_type: SqlDataType::DOUBLE,
        non_concise_type: SqlDataType::DOUBLE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(15),
        precision: Some(15),
        octet_length: Some(8),
        fixed_bytes_length: Some(8),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        simple_type_info: None,
    };
    pub const STRING: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::EXT_W_VARCHAR,
        non_concise_type: SqlDataType::EXT_W_VARCHAR,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: Some(65535),
        precision: Some(65535),
        octet_length: Some(65535),
        fixed_bytes_length: Some(65535),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: Some(65535),
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: Some(65535),
        simple_type_info: None,
    };
    pub const VARCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "varchar",
        sql_type: SqlDataType::VARCHAR,
        non_concise_type: SqlDataType::VARCHAR,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: Some(65535),
        precision: Some(65535),
        octet_length: Some(65535),
        fixed_bytes_length: Some(65535),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: Some(65535),
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: Some(65535),
        simple_type_info: None,
    };
    pub const OBJECT: BsonTypeInfo = BsonTypeInfo {
        type_name: "object",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const ARRAY: BsonTypeInfo = BsonTypeInfo {
        type_name: "array",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const BINDATA: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        sql_type: SqlDataType::EXT_BINARY,
        non_concise_type: SqlDataType::EXT_BINARY,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const UNDEFINED: BsonTypeInfo = BsonTypeInfo {
        type_name: "undefined",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::new(20, 20, 20),
    };
    pub const OBJECTID: BsonTypeInfo = BsonTypeInfo {
        type_name: "objectId",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: Some(24),
        octet_length: Some(24),
        fixed_bytes_length: Some(24),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(true),
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::new(34, 34, 34),
    };
    pub const BOOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "bool",
        sql_type: SqlDataType::EXT_BIT,
        non_concise_type: SqlDataType::EXT_BIT,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: Some(1),
        octet_length: Some(1),
        fixed_bytes_length: Some(1),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const DATE: BsonTypeInfo = BsonTypeInfo {
        type_name: "date",
        sql_type: SqlDataType::TIMESTAMP,
        non_concise_type: SqlDataType::DATETIME,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(3),
        precision: Some(23),
        octet_length: Some(16),
        fixed_bytes_length: Some(16),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: Some(3),
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const NULL: BsonTypeInfo = BsonTypeInfo {
        type_name: "null",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const REGEX: BsonTypeInfo = BsonTypeInfo {
        type_name: "regex",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const DBPOINTER: BsonTypeInfo = BsonTypeInfo {
        type_name: "dbPointer",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const JAVASCRIPT: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascript",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const SYMBOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "symbol",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const JAVASCRIPTWITHSCOPE: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascriptWithScope",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const INT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        sql_type: SqlDataType::INTEGER,
        non_concise_type: SqlDataType::INTEGER,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        precision: Some(10),
        octet_length: Some(65535),
        fixed_bytes_length: Some(65535),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        simple_type_info: None,
    };
    pub const TIMESTAMP: BsonTypeInfo = BsonTypeInfo {
        type_name: "timestamp",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::new(68, 68, 68),
    };
    pub const LONG: BsonTypeInfo = BsonTypeInfo {
        type_name: "long",
        sql_type: SqlDataType::EXT_BIG_INT,
        non_concise_type: SqlDataType::EXT_BIG_INT,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        precision: Some(19),
        octet_length: Some(8),
        fixed_bytes_length: Some(8),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        simple_type_info: None,
    };
    pub const DECIMAL: BsonTypeInfo = BsonTypeInfo {
        type_name: "decimal",
        sql_type: SqlDataType::DECIMAL,
        non_concise_type: SqlDataType::DECIMAL,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: Some(34),
        octet_length: Some(16),
        fixed_bytes_length: Some(16),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const MINKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "minKey",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::new(14, 14, 14),
    };
    pub const MAXKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "maxKey",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::new(14, 14, 14),
    };
    pub const BSON: BsonTypeInfo = BsonTypeInfo {
        type_name: "bson",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        non_concise_type: SqlDataType::UNKNOWN_TYPE,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };

    pub fn sql_type(&self, type_mode: TypeMode) -> SqlDataType {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().sql_type
        } else {
            self.sql_type
        }
    }

    pub fn non_concise_type(&self, type_mode: TypeMode) -> SqlDataType {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().non_concise_type
        } else {
            self.non_concise_type
        }
    }

    pub fn precision(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().precision
        } else {
            self.precision
        }
    }

    pub fn octet_length(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().octet_length
        } else {
            self.octet_length
        }
    }

    pub fn fixed_bytes_length(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().fixed_bytes_length
        } else {
            self.fixed_bytes_length
        }
    }
}

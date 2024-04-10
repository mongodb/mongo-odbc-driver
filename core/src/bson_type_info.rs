use definitions::{SqlCode, SqlDataType};

pub const MAX_STRING_SIZE: u16 = u16::MAX;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TypeMode {
    Standard,
    Simple,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsonTypeInfo {
    // This is the String bson typename as reported by the $type function in MQL
    pub type_name: &'static str,
    // This is the SqlDataType integer as per the ODBC spec
    pub sql_type: SqlDataType,
    // non_concise_type is also a SqlDataType integer, but it can differ from the sql_type for some
    // types, e.g. TIMESTAMP
    pub non_concise_type: SqlDataType,
    // An integer representing an enumeration of different searchability values:
    // SQL_SEARCHABLE = 3; SQL_PRED_BASIC = 2; SQL_PRED_NONE = 0;
    pub searchable: i32,
    // A boolean value reporting if a type is case_sensitive. True for Char-types, false for others
    pub is_case_sensitive: bool,
    // A boolean dictating if the data type has predefined fixed precision and scale (which are data source-specific),
    // such as a money data type. Note true for DOUBLE, where scale can differ
    pub fixed_prec_scale: bool,
    // Scale for a datatype. If the datatype does not have fixed scale, this represents the largest
    // possible scale.
    pub scale: Option<u16>,
    // Precision for the data type. For char-data, this represents the largest possible character length, for numerical
    // data, this represents the maximum number of decimal places.
    pub precision: Option<u16>,
    // Maximum number of bytes (octets) for a given data type
    pub octet_length: Option<u16>,
    // Number of bytes in a type that has fixed length, such as INT or DOUBLE
    pub fixed_bytes_length: Option<u16>,
    // Prefix used for a literal of this type, such as ' for a char-type
    pub literal_prefix: Option<&'static str>,
    // Suffix used for a literal of this type, such as ' for a char-type
    pub literal_suffix: Option<&'static str>,
    // Additional info that is currently only applied to Dates
    pub sql_code: Option<SqlCode>,
    // Represents if this type is automatically unique in a given column, such as OBJECTID
    pub is_auto_unique_value: Option<bool>,
    // A bool if data is unsigned, None if not applicable, such as a char-type
    pub is_unsigned: Option<bool>,
    // If the data type is an approximate numeric type, this contains the value Some(2) to indicate
    // that COLUMN_SIZE specifies a number of bits. For exact numeric types, this column contains
    // the value Some(10) to indicate that COLUMN_SIZE specifies a number of decimal digits.
    // Otherwise, this is None.
    pub num_prec_radix: Option<u16>,
    // This is the type info we use when simple_type_mode is true. This is a convenience mode for
    // BI tools where BSON types not directly representable as SQL data are rendered as extended
    // json strings.
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
            sql_type: SqlDataType::SQL_WVARCHAR,
            non_concise_type: SqlDataType::SQL_WVARCHAR,
            precision: Some(precision),
            octet_length: Some(octet_length),
            fixed_bytes_length: Some(fixed_bytes_length),
        })
    }

    const fn default() -> Option<Self> {
        Some(Self {
            sql_type: SqlDataType::SQL_WVARCHAR,
            non_concise_type: SqlDataType::SQL_WVARCHAR,
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
        sql_type: SqlDataType::SQL_DOUBLE,
        non_concise_type: SqlDataType::SQL_DOUBLE,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: Some(15),
        precision: Some(15),
        octet_length: Some(8),
        fixed_bytes_length: Some(8),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(2),
        simple_type_info: None,
    };
    pub const STRING: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_WVARCHAR,
        non_concise_type: SqlDataType::SQL_WVARCHAR,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = VARCHAR
    pub const VARCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "varchar",
        sql_type: SqlDataType::SQL_VARCHAR,
        non_concise_type: SqlDataType::SQL_VARCHAR,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        precision: Some(MAX_STRING_SIZE),
        octet_length: None,
        fixed_bytes_length: None,
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const OBJECT: BsonTypeInfo = BsonTypeInfo {
        type_name: "object",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_BINARY,
        non_concise_type: SqlDataType::SQL_BINARY,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_BIT,
        non_concise_type: SqlDataType::SQL_BIT,
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
        sql_type: SqlDataType::SQL_TYPE_TIMESTAMP,
        non_concise_type: SqlDataType::SQL_DATETIME,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(3),
        precision: Some(23),
        octet_length: Some(16),
        fixed_bytes_length: Some(16),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: Some(SqlCode::SQL_CODE_TIMESTAMP),
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        simple_type_info: None,
    };
    pub const NULL: BsonTypeInfo = BsonTypeInfo {
        type_name: "null",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        simple_type_info: SimpleTypeInfo::new(4, 4, 4),
    };
    pub const REGEX: BsonTypeInfo = BsonTypeInfo {
        type_name: "regex",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_INTEGER,
        non_concise_type: SqlDataType::SQL_INTEGER,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        precision: Some(10),
        octet_length: Some(4),
        fixed_bytes_length: Some(4),
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_BIGINT,
        non_concise_type: SqlDataType::SQL_BIGINT,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: None,
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const MINKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "minKey",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: SqlDataType::SQL_UNKNOWN_TYPE,
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

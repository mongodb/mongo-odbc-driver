use definitions::{SqlCode, SqlDataType};

pub const MAX_STRING_SIZE: u16 = u16::MAX;

#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(C)]
pub enum TypeMode {
    Standard,
    Simple,
}

/// make_default_attr_func creates an anonymous function that takes a single
/// wildcard argument and returns the provided default value. This is useful
/// for setting certain attributes for BsonTypeInfo which are defined as
/// functions.
macro_rules! make_default_attr_func {
    ($default_value:expr) => {
        |_| $default_value
    };
}

#[non_exhaustive]
#[derive(Debug, Eq, Clone)]
pub struct BsonTypeInfo {
    // This is the String bson typename as reported by the $type function in MQL
    pub type_name: &'static str,
    // This is the SqlDataType integer as per the ODBC spec
    pub sql_type: SqlDataType,
    // non_concise_type is an optional SqlDataType integer. It can differ from the sql_type for some
    // types, e.g. TIMESTAMP. When this value is the same as sql_type, it is set to None.
    pub non_concise_type: Option<SqlDataType>,
    // An integer representing an enumeration of different searchability values:
    // SQL_SEARCHABLE = 3; SQL_PRED_BASIC = 2; SQL_PRED_NONE = 0;
    pub searchable: i32,
    // A boolean value reporting if a type is case_sensitive. True for Char-types, false for others
    pub is_case_sensitive: bool,
    // A boolean dictating if the data type has predefined fixed precision and scale (which are data source-specific),
    // such as a money data type. Note true for DOUBLE, where scale can differ.
    pub fixed_prec_scale: bool,
    // Scale for a datatype. If the datatype does not have fixed scale, this represents the largest
    // possible scale.
    pub scale: Option<u16>,
    // The maximum or actual character length of a character string or binary data type. It is the
    // maximum character length for a fixed-length data type, or the actual character length for a
    // variable-length data type. Its value always excludes the null-termination byte that ends the
    // character string. This is a function since a BI tool may set a maximum string length. For
    // most types, the function argument is ignored and a default value is returned; for the String
    // type, if a max string length is set, it is argued to the function and returned.
    pub length: fn(Option<u16>) -> Option<u16>,
    // For a numeric data type denotes the applicable precision. For data types SQL_TYPE_TIME,
    // SQL_TYPE_TIMESTAMP, and all the interval data types that represent a time interval, its value
    // is the applicable precision of the fractional seconds component.
    pub precision: Option<u16>,
    // The length, in bytes, of a character string or binary data type. This is a function since a
    // BI tool may set a maximum string length. For most types, the function argument is ignored and
    // a default value is returned; for the String type, if a max string length is set, it is argued
    // to the function and returned.
    pub char_octet_length: fn(Option<u16>) -> Option<u16>,
    // The transfer octet length of a column is the maximum number of bytes returned to the
    // application when data is transferred to its default C data type.
    pub transfer_octet_length: Option<u16>,
    // The maximum number of characters needed to display data in character form. This is a function
    // since a BI tool may set a maximum string length. For most types, the function argument is
    // ignored and a default value is returned; for the String type, if a max string length is set,
    // it is argued to the function and returned.
    pub display_size: fn(Option<u16>) -> Option<u16>,
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
    // The decimal digits of decimal and numeric data types is defined as the maximum number of
    // digits to the right of the decimal point, or the scale of the data.
    // For approximate floating-point number columns or parameters, the scale is undefined because
    // the number of digits to the right of the decimal point is not fixed.
    // For datetime or interval data that contains a seconds component, the decimal digits is
    // defined as the number of digits to the right of the decimal point in the seconds component of
    // the data.
    // Descriptor field corresponding to decimal digits:
    //      - All exact numeric types: SQL_DESC_SCALE
    //      - All datetime types and interval types : SQL_DESC_PRECISION
    //      - All other types : Not applicable
    pub decimal_digit: Option<u16>,
    // The column (or parameter) size of numeric data types is defined as the maximum number of
    // digits used by the data type of the column or parameter, or the precision of the data. For
    // character types, this is the length in characters of the data; for binary data types, column
    // size is defined as the length in bytes of the data. For the time, timestamp, and all interval
    // data types, this is the number of characters in the character representation of this data.
    // Descriptor field corresponding to decimal digits:
    //      - All numeric types except SQL_BIT: SQL_DESC_PRECISION
    //      - All other types: SQL_DESC_LENGTH
    // This is a function since a BI tool may set a maximum string length. For most types, the
    // function argument is ignored and a default value is returned; for the String type, if a max
    // string length is set, it is argued to the function and returned.
    pub column_size: fn(Option<u16>) -> Option<u16>,
    // This is the type info we use when simple_type_mode is true. This is a convenience mode for
    // BI tools where BSON types not directly representable as SQL data are rendered as extended
    // json strings.
    pub simple_type_info: Option<SimpleTypeInfo>,
}

impl PartialEq for BsonTypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.sql_type == other.sql_type
            && self.non_concise_type == other.non_concise_type
            && self.searchable == other.searchable
            && self.is_case_sensitive == other.is_case_sensitive
            && self.fixed_prec_scale == other.fixed_prec_scale
            && self.scale == other.scale
            && self.precision == other.precision
            && self.transfer_octet_length == other.transfer_octet_length
            && self.literal_prefix == other.literal_prefix
            && self.literal_suffix == other.literal_suffix
            && self.sql_code == other.sql_code
            && self.is_auto_unique_value == other.is_auto_unique_value
            && self.is_unsigned == other.is_unsigned
            && self.num_prec_radix == other.num_prec_radix
            && self.decimal_digit == other.decimal_digit
            && self.simple_type_info == other.simple_type_info
    }
}

#[derive(Debug, Eq, Clone)]
pub struct SimpleTypeInfo {
    pub sql_type: SqlDataType,
    pub non_concise_type: Option<SqlDataType>,
    pub length: fn(Option<u16>) -> Option<u16>,
    pub transfer_octet_length: Option<u16>,
    pub display_size: fn(Option<u16>) -> Option<u16>,
}

impl PartialEq for SimpleTypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.sql_type == other.sql_type
            && self.non_concise_type == other.non_concise_type
            && self.transfer_octet_length == other.transfer_octet_length
    }
}

macro_rules! new_simple_type_info {
    ($length:expr, $transfer_octet_length:expr, $display_size:expr) => {
        Some(SimpleTypeInfo {
            sql_type: SqlDataType::SQL_WVARCHAR,
            non_concise_type: None,
            length: make_default_attr_func!(Some($length)),
            transfer_octet_length: Some($transfer_octet_length),
            display_size: make_default_attr_func!(Some($display_size)),
        })
    };
}
impl SimpleTypeInfo {
    const fn default() -> Option<Self> {
        Some(Self {
            sql_type: SqlDataType::SQL_WVARCHAR,
            non_concise_type: None,
            length: |max_string_length| max_string_length,
            transfer_octet_length: None,
            display_size: |max_string_length| max_string_length,
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
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(15),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(8),
        display_size: make_default_attr_func!(Some(24)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(2),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(15)),
        simple_type_info: None,
    };
    // This is just here to support any tools that attempt to cast to FLOAT.
    // FLOAT appears to be identical to DOUBLE other than that a precsion can be specified.
    pub const FLOAT: BsonTypeInfo = BsonTypeInfo {
        type_name: "double",
        sql_type: SqlDataType::SQL_FLOAT,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(15),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(8),
        display_size: make_default_attr_func!(Some(24)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(2),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(15)),
        simple_type_info: None,
    };
    // We support REAL for any types that use it. We map it to "double" as the mongo name for the
    // purposes of CAST in the syntax (e.g., it will generate CAST(x AS DOUBLE) in Direct Query).
    pub const REAL: BsonTypeInfo = BsonTypeInfo {
        type_name: "double",
        sql_type: SqlDataType::SQL_REAL,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(7),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(4),
        display_size: make_default_attr_func!(Some(14)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(2),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(7)),
        simple_type_info: None,
    };
    // This represents the literal mongodb string type. Other bson type
    // info mapping to "string" are aliases for the benefits of bi tools.
    pub const STRING: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_WVARCHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: |max_string_length| max_string_length,
        precision: None,
        char_octet_length: |max_string_length| max_string_length,
        transfer_octet_length: None,
        display_size: |max_string_length| max_string_length,
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: |max_string_length| max_string_length,
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = CHAR.
    pub const CHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_CHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        precision: None,
        char_octet_length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = LONGVARCHAR. This comes up when scrolling through large string
    // datasets with direct query enabled.
    pub const LONGVARCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_LONGVARCHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        precision: None,
        char_octet_length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = VARCHAR
    pub const VARCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "varchar",
        sql_type: SqlDataType::SQL_VARCHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        precision: None,
        char_octet_length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = WLONGVARCHAR. This comes up when scrolling through large string
    // datasets with direct query enabled.
    pub const WLONGVARCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_WLONGVARCHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        precision: None,
        char_octet_length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting
    // to text in Power BI because they look for a type that is specifically
    // sqltype = WCHAR
    pub const WCHAR: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::SQL_WCHAR,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: true,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        precision: None,
        char_octet_length: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(MAX_STRING_SIZE)),
        simple_type_info: None,
    };
    pub const OBJECT: BsonTypeInfo = BsonTypeInfo {
        type_name: "object",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const ARRAY: BsonTypeInfo = BsonTypeInfo {
        type_name: "array",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const BINDATA: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        sql_type: SqlDataType::SQL_BINARY,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    // We just map LONGVARBINARY to BINARY since mongo only has one binary type.
    pub const LONGVARBINARY: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        sql_type: SqlDataType::SQL_LONGVARBINARY,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    // We just map VARBINARY to BINARY since mongo only has one binary type.
    pub const VARBINARY: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        sql_type: SqlDataType::SQL_VARBINARY,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const UNDEFINED: BsonTypeInfo = BsonTypeInfo {
        type_name: "undefined",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: new_simple_type_info!(20, 20 * 4, 20),
    };
    pub const OBJECTID: BsonTypeInfo = BsonTypeInfo {
        type_name: "objectId",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(24),
        display_size: make_default_attr_func!(Some(24)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(true),
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(Some(24)),
        simple_type_info: new_simple_type_info!(35, 35 * 4, 35),
    };
    pub const BOOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "bool",
        sql_type: SqlDataType::SQL_BIT,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: Some(0),
        length: make_default_attr_func!(Some(1)),
        precision: Some(1),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(1),
        display_size: make_default_attr_func!(Some(1)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(1)),
        simple_type_info: None,
    };
    pub const DATE: BsonTypeInfo = BsonTypeInfo {
        type_name: "date",
        sql_type: SqlDataType::SQL_TYPE_TIMESTAMP,
        non_concise_type: Some(SqlDataType::SQL_DATETIME),
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: None,
        length: make_default_attr_func!(Some(23)),
        precision: Some(3),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(16),
        display_size: make_default_attr_func!(Some(23)),
        literal_prefix: Some("'"),
        literal_suffix: Some("'"),
        sql_code: Some(SqlCode::SQL_CODE_TIMESTAMP),
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: Some(3),
        column_size: make_default_attr_func!(Some(23)),
        simple_type_info: None,
    };
    pub const NULL: BsonTypeInfo = BsonTypeInfo {
        type_name: "null",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_SEARCHABLE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: new_simple_type_info!(4, 4 * 4, 4),
    };
    pub const REGEX: BsonTypeInfo = BsonTypeInfo {
        type_name: "regex",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const DBPOINTER: BsonTypeInfo = BsonTypeInfo {
        type_name: "dbPointer",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const JAVASCRIPT: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascript",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const SYMBOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "symbol",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const JAVASCRIPTWITHSCOPE: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascriptWithScope",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    // This is essentially here just to support Direct Query casting for small integers. Since int
    // is the smallest integer type in Mongo, we map all small integers to int.
    pub const TINYINT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        sql_type: SqlDataType::SQL_TINYINT,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(3),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(1),
        display_size: make_default_attr_func!(Some(4)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(3)),
        simple_type_info: None,
    };
    // This is essentially here just to support Direct Query casting for small integers. Since int
    // is the smallest integer type in Mongo, we map all small integers to int.
    pub const SMALLINT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        sql_type: SqlDataType::SQL_SMALLINT,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(5),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(4),
        display_size: make_default_attr_func!(Some(6)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(5)),
        simple_type_info: None,
    };
    pub const INT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        sql_type: SqlDataType::SQL_INTEGER,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(10),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(4),
        display_size: make_default_attr_func!(Some(11)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(10)),
        simple_type_info: None,
    };
    pub const TIMESTAMP: BsonTypeInfo = BsonTypeInfo {
        type_name: "timestamp",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: new_simple_type_info!(68, 68 * 4, 68),
    };
    pub const LONG: BsonTypeInfo = BsonTypeInfo {
        type_name: "long",
        sql_type: SqlDataType::SQL_BIGINT,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: true,
        scale: Some(0),
        length: make_default_attr_func!(None),
        precision: Some(20),
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: Some(8),
        display_size: make_default_attr_func!(Some(20)),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: Some(10),
        decimal_digit: Some(0),
        column_size: make_default_attr_func!(Some(20)),
        simple_type_info: None,
    };
    // MONGO_DECIMAL is our mongo decimal128 floating point type. We use SQL_UNKNOWN_TYPE to ensure
    // that our uses must cast Mongo Decimals to something else to retrieve, or they are displayed
    // as json strings by default.
    pub const MONGO_DECIMAL: BsonTypeInfo = BsonTypeInfo {
        type_name: "decimal",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: Some(false),
        is_unsigned: Some(false),
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: SimpleTypeInfo::default(),
    };
    pub const MINKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "minKey",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: new_simple_type_info!(14, 14 * 4, 14),
    };
    pub const MAXKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "maxKey",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_BASIC,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
        simple_type_info: new_simple_type_info!(14, 14 * 4, 14),
    };
    pub const BSON: BsonTypeInfo = BsonTypeInfo {
        type_name: "bson",
        sql_type: SqlDataType::SQL_UNKNOWN_TYPE,
        non_concise_type: None,
        searchable: SQL_PRED_NONE,
        is_case_sensitive: false,
        fixed_prec_scale: false,
        scale: None,
        length: make_default_attr_func!(None),
        precision: None,
        char_octet_length: make_default_attr_func!(None),
        transfer_octet_length: None,
        display_size: make_default_attr_func!(None),
        literal_prefix: None,
        literal_suffix: None,
        sql_code: None,
        is_auto_unique_value: None,
        is_unsigned: None,
        num_prec_radix: None,
        decimal_digit: None,
        column_size: make_default_attr_func!(None),
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
            let simple_type_info = self.simple_type_info.clone().unwrap();
            simple_type_info
                .non_concise_type
                .unwrap_or(simple_type_info.sql_type)
        } else {
            self.non_concise_type.unwrap_or(self.sql_type)
        }
    }

    pub fn precision(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            None
        } else {
            self.precision
        }
    }

    pub fn length(&self, type_mode: TypeMode, max_string_length: Option<u16>) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            (self.simple_type_info.clone().unwrap().length)(max_string_length)
        } else {
            (self.length)(max_string_length)
        }
    }

    pub fn transfer_octet_length(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            self.simple_type_info.clone().unwrap().transfer_octet_length
        } else {
            self.transfer_octet_length
        }
    }

    pub fn char_octet_length(
        &self,
        type_mode: TypeMode,
        max_string_length: Option<u16>,
    ) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            (self.simple_type_info.clone().unwrap().length)(max_string_length)
        } else {
            (self.char_octet_length)(max_string_length)
        }
    }

    pub fn display_size(&self, type_mode: TypeMode, max_string_length: Option<u16>) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            (self.simple_type_info.clone().unwrap().display_size)(max_string_length)
        } else {
            (self.display_size)(max_string_length)
        }
    }

    pub fn decimal_digit(&self, type_mode: TypeMode) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            None
        } else {
            self.decimal_digit
        }
    }

    pub fn column_size(&self, type_mode: TypeMode, max_string_length: Option<u16>) -> Option<u16> {
        if type_mode == TypeMode::Simple && self.simple_type_info.is_some() {
            (self.simple_type_info.clone().unwrap().length)(max_string_length)
        } else {
            (self.column_size)(max_string_length)
        }
    }
}

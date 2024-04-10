use crate::{
    bson_type_info::SQL_PRED_BASIC, col_metadata::MongoColMetadata, conn::MongoConnection,
    err::Result, stmt::MongoStatement, BsonTypeInfo, Error, TypeMode,
};
use bson::Bson;
use definitions::{Nullability, SqlCode, SqlDataType};

use lazy_static::lazy_static;

// this type is needed for backwards compatibility when the application sets odbc version to 2.
// when all types are requested from SQLGetTypeInfo, DATE and LEGACY_DATE should both be returned.
const LEGACY_DATE: BsonTypeInfo = BsonTypeInfo {
    type_name: "date",
    sql_type: SqlDataType::SQL_TIMESTAMP,
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

// order of array is by SqlDataType, since that is the ordering of the
// SQLGetTypeInfo result set according to the spec
const DATA_TYPES: [BsonTypeInfo; 23] = [
    BsonTypeInfo::STRING,              // SqlDataType(-9)
    BsonTypeInfo::BOOL,                // SqlDataType(-7)
    BsonTypeInfo::LONG,                // SqlDataType(-5)
    BsonTypeInfo::BINDATA,             // SqlDataType(-2)
    BsonTypeInfo::ARRAY,               // SqlDataType(0)
    BsonTypeInfo::BSON,                // SqlDataType(0)
    BsonTypeInfo::DBPOINTER,           // SqlDataType(0)
    BsonTypeInfo::DECIMAL,             // SqlDataType(0)
    BsonTypeInfo::JAVASCRIPT,          // SqlDataType(0)
    BsonTypeInfo::JAVASCRIPTWITHSCOPE, // SqlDataType(0)
    BsonTypeInfo::MAXKEY,              // SqlDataType(0)
    BsonTypeInfo::MINKEY,              // SqlDataType(0)
    BsonTypeInfo::NULL,                // SqlDataType(0)
    BsonTypeInfo::OBJECT,              // SqlDataType(0)
    BsonTypeInfo::OBJECTID,            // SqlDataType(0)
    BsonTypeInfo::SYMBOL,              // SqlDataType(0)
    BsonTypeInfo::TIMESTAMP,           // SqlDataType(0)
    BsonTypeInfo::UNDEFINED,           // SqlDataType(0)
    BsonTypeInfo::INT,                 // SqlDataType(4)
    BsonTypeInfo::DOUBLE,              // SqlDataType(8)
    LEGACY_DATE,                       // SqlDataType(11)
    BsonTypeInfo::VARCHAR,             // SqlDataType(12)
    BsonTypeInfo::DATE,                // SqlDataType(93)
];

lazy_static! {
    pub static ref TYPES_INFO_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "DATATYPE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "LITERAL_PREFIX".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "LITERAL_SUFFIX".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "CREATE_PARAMS".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "CASE_SENSITIVE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "SEARCHABLE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "UNSIGNED_ATTRIBUTE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FIXED_PREC_SCALE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "AUTO_UNIQUE_VALUE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "LOCAL_TYPE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "MINIMUM_SCALE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "MAXIMUM_SCALE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "INTERVAL_PRECISION".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
    ];
}

#[derive(Debug)]
#[repr(C)]
pub struct MongoTypesInfo {
    current_type_index: usize,
    sql_data_type: SqlDataType,
    type_mode: TypeMode,
}

impl MongoTypesInfo {
    pub fn new(sql_data_type: SqlDataType, type_mode: TypeMode) -> MongoTypesInfo {
        MongoTypesInfo {
            current_type_index: 0,
            sql_data_type,
            type_mode,
        }
    }
}

impl MongoStatement for MongoTypesInfo {
    // iterate through the list, searching for the next "valid" data type.
    // a type is valid if its sql type matches the desired sql type, or if we are getting all types.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        loop {
            self.current_type_index += 1;
            if self.current_type_index > DATA_TYPES.len()
                || DATA_TYPES[self.current_type_index - 1].sql_type(self.type_mode)
                    == self.sql_data_type
                || self.sql_data_type == SqlDataType::SQL_UNKNOWN_TYPE
            {
                break;
            }
        }
        Ok((self.current_type_index <= DATA_TYPES.len(), vec![]))
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // 1 -> TYPE_NAME
        // 2 -> DATA_TYPE
        // 3 -> COLUMN_SIZE
        // 4 -> LITERAL_PREFIX
        // 5 -> LITERAL_SUFFIX
        // 6 -> CREATE_PARAMS
        // 7 -> NULLABLE
        // 8 -> CASE_SENSITIVE
        // 9 -> SEARCHABLE
        // 10 -> UNSIGNED_ATTRIBUTE
        // 11 -> FIXED_PREC_SCALE
        // 12 -> AUTO_UNIQUE_VALUE
        // 13 -> LOCAL_TYPE_NAME
        // 14 -> MINIMUM_SCALE
        // 15 -> MAXIMUM_SCALE
        // 16 -> SQL_DATA_TYPE
        // 17 -> SQL_DATETIME_SUB
        // 18 -> NUM_PREC_RADIX
        // 19 -> INTERVAL_PRECISION
        // Fails if the first row as not been retrieved (next must be called at least once before getValue).
        if self.current_type_index == 0 {
            return Err(Error::InvalidCursorState);
        }

        match DATA_TYPES.get(self.current_type_index - 1) {
            Some(type_info) => Ok(Some(match col_index {
                1 | 13 => Bson::String(type_info.type_name.to_string()),
                2 | 16 => Bson::Int32(type_info.sql_type(self.type_mode) as i32),
                3 => match type_info.precision(self.type_mode) {
                    Some(precision) => Bson::Int32(precision as i32),
                    None => Bson::Null,
                },
                4 => match type_info.literal_prefix {
                    Some(prefix) => Bson::String(prefix.to_string()),
                    _ => Bson::Null,
                },
                5 => match type_info.literal_suffix {
                    Some(suffix) => Bson::String(suffix.to_string()),
                    _ => Bson::Null,
                },
                6 => Bson::Null,
                7 => Bson::Int32(Nullability::SQL_NULLABLE as i32),
                8 => Bson::Int32(type_info.is_case_sensitive as i32),
                9 => Bson::Int32(type_info.searchable),
                10 => match type_info.is_unsigned {
                    Some(signed) => Bson::Int32(signed as i32),
                    _ => Bson::Null,
                },
                11 => Bson::Int32(type_info.fixed_prec_scale as i32),
                12 => match type_info.is_auto_unique_value {
                    Some(is_auto_unique_value) => Bson::Int32(is_auto_unique_value as i32),
                    _ => Bson::Null,
                },
                14 | 15 => match type_info.scale {
                    Some(scale) => Bson::Int32(scale as i32),
                    None => Bson::Null,
                },
                17 => match type_info.sql_code {
                    Some(subcode) => Bson::Int32(subcode as i32),
                    _ => Bson::Null,
                },
                18 => match type_info.num_prec_radix {
                    Some(num_prec_radix) => Bson::Int32(num_prec_radix as i32),
                    _ => Bson::Null,
                },
                19 => Bson::Null,
                _ => return Err(Error::ColIndexOutOfBounds(col_index)),
            })),
            None => Err(Error::ColIndexOutOfBounds(col_index)),
        }
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &TYPES_INFO_METADATA
    }
}

use crate::{
    col_metadata::MongoColMetadata,
    conn::MongoConnection,
    definitions::SqlDataType,
    err::Result,
    type_info::{SimpleBsonTypeInfo, StandardBsonTypeInfo},
    stmt::MongoStatement,
    BsonTypeInfo, Error, TypeMode,
};
use bson::Bson;
use odbc_sys::Nullability;

use crate::type_info::TypeInfoFields;
use lazy_static::lazy_static;

const STANDARD_DATA_TYPES: [TypeInfoFields; 21] = [
    StandardBsonTypeInfo::STRING.type_info_fields, // SqlDataType(-9)
    StandardBsonTypeInfo::BOOL.type_info_fields,   // SqlDataType(-7)
    StandardBsonTypeInfo::LONG.type_info_fields,   // SqlDataType(-5)
    StandardBsonTypeInfo::BINDATA.type_info_fields, // SqlDataType(-2)
    StandardBsonTypeInfo::ARRAY.type_info_fields,  // SqlDataType(0)
    StandardBsonTypeInfo::BSON.type_info_fields,   // SqlDataType(0)
    StandardBsonTypeInfo::DBPOINTER.type_info_fields, //SqlDataType(0)
    StandardBsonTypeInfo::DECIMAL.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::JAVASCRIPT.type_info_fields, //SqlDataType(0)
    StandardBsonTypeInfo::JAVASCRIPTWITHSCOPE.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::MAXKEY.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::MINKEY.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::NULL.type_info_fields,   // SqlDataType(0)
    StandardBsonTypeInfo::OBJECT.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::OBJECTID.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::SYMBOL.type_info_fields, //SqlDataType(0)
    StandardBsonTypeInfo::TIMESTAMP.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::UNDEFINED.type_info_fields, // SqlDataType(0)
    StandardBsonTypeInfo::INT.type_info_fields,    // SqlDataType(4)
    StandardBsonTypeInfo::DOUBLE.type_info_fields, // SqlDataType(8)
    StandardBsonTypeInfo::DATE.type_info_fields,   // SqlDataType(93)
];

const SIMPLE_DATA_TYPES: [TypeInfoFields; 21] = [
    SimpleBsonTypeInfo::STRING.type_info_fields, // SqlDataType(-9)
    SimpleBsonTypeInfo::BOOL.type_info_fields,   // SqlDataType(-7)
    SimpleBsonTypeInfo::LONG.type_info_fields,   // SqlDataType(-5)
    SimpleBsonTypeInfo::BINDATA.type_info_fields, // SqlDataType(-2)
    SimpleBsonTypeInfo::ARRAY.type_info_fields,  // SqlDataType(0)
    SimpleBsonTypeInfo::BSON.type_info_fields,   // SqlDataType(0)
    SimpleBsonTypeInfo::DBPOINTER.type_info_fields, //SqlDataType(0)
    SimpleBsonTypeInfo::DECIMAL.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::JAVASCRIPT.type_info_fields, //SqlDataType(0)
    SimpleBsonTypeInfo::JAVASCRIPTWITHSCOPE.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::MAXKEY.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::MINKEY.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::NULL.type_info_fields,   // SqlDataType(0)
    SimpleBsonTypeInfo::OBJECT.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::OBJECTID.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::SYMBOL.type_info_fields, //SqlDataType(0)
    SimpleBsonTypeInfo::TIMESTAMP.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::UNDEFINED.type_info_fields, // SqlDataType(0)
    SimpleBsonTypeInfo::INT.type_info_fields,    // SqlDataType(4)
    SimpleBsonTypeInfo::DOUBLE.type_info_fields, // SqlDataType(8)
    SimpleBsonTypeInfo::DATE.type_info_fields,   // SqlDataType(93)
];

lazy_static! {
    pub static ref STANDARD_TYPES_INFO_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DATATYPE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LITERAL_PREFIX".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LITERAL_SUFFIX".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "CREATE_PARAMS".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "CASE_SENSITIVE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SEARCHABLE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "UNSIGNED_ATTRIBUTE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FIXED_PREC_SCALE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "AUTO_UNIQUE_VALUE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LOCAL_TYPE_NAME".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "MINIMUM_SCALE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "MAXIMUM_SCALE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "INTERVAL_PRECISION".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
    ];
    pub static ref SIMPLE_TYPES_INFO_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DATATYPE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LITERAL_PREFIX".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LITERAL_SUFFIX".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "CREATE_PARAMS".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "CASE_SENSITIVE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SEARCHABLE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "UNSIGNED_ATTRIBUTE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FIXED_PREC_SCALE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "AUTO_UNIQUE_VALUE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "LOCAL_TYPE_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "MINIMUM_SCALE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "MAXIMUM_SCALE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "INTERVAL_PRECISION".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NULLABLE
        ),
    ];
}

#[derive(Debug)]
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
        let data_types = match self.type_mode {
            TypeMode::Standard => STANDARD_DATA_TYPES,
            TypeMode::Simple => SIMPLE_DATA_TYPES,
        };

        loop {
            self.current_type_index += 1;
            if self.current_type_index > data_types.len()
                || data_types[self.current_type_index - 1].sql_type == self.sql_data_type
                || self.sql_data_type == SqlDataType::UNKNOWN_TYPE
            {
                break;
            }
        }
        Ok((self.current_type_index <= data_types.len(), vec![]))
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

        let data_type = match self.type_mode {
            TypeMode::Standard => STANDARD_DATA_TYPES.get(self.current_type_index - 1),
            TypeMode::Simple => SIMPLE_DATA_TYPES.get(self.current_type_index - 1),
        };

        match data_type {
            Some(type_info) => Ok(Some(match col_index {
                1 | 13 => Bson::String(type_info.type_name.to_string()),
                2 | 16 => Bson::Int32(type_info.sql_type as i32),
                3 => match type_info.precision {
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
                7 => Bson::Int32(Nullability::NULLABLE.0 as i32),
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
                    Some(subcode) => Bson::Int32(subcode),
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
        match self.type_mode {
            TypeMode::Standard => &STANDARD_TYPES_INFO_METADATA,
            TypeMode::Simple => &SIMPLE_TYPES_INFO_METADATA,
        }
    }
}

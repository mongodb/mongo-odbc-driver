use crate::{
    add_diag_with_function,
    errors::ODBCError,
    handles::definitions::{CachedData, MongoHandle, Statement},
};
use bson::{spec::BinarySubtype, Bson, UuidRepresentation};
use chrono::{
    offset::Utc, DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike,
};
use cstr::{
    write_binary_slice_to_buffer, write_fixed_data, write_string_slice_to_buffer,
    write_wstring_slice_to_buffer, WideChar,
};
use definitions::{
    CDataType, Char, Date, Integer, Len, Pointer, SmallInt, SqlReturn, Time, Timestamp, USmallInt,
};
use regex::Regex;
use serde_json::{json, Value};
use std::{mem::size_of, str::FromStr};

const DOUBLE: &str = "Double";
const INT32: &str = "Int32";
const INT64: &str = "Int64";
const UINT32: &str = "UInt32";
const UINT64: &str = "UInt64";
const BIT: &str = "Bit";
const DATETIME: &str = "DateTime";
const GUID: &str = "GUID";

type Result<T> = std::result::Result<T, ODBCError>;

/// IntoCData is just used for adding methods to bson::Bson.
trait IntoCData {
    fn to_json(self, uuid_repr: Option<UuidRepresentation>) -> String;
    fn to_json_val(self, uuid_repr: Option<UuidRepresentation>) -> Value;
    fn to_binary(self, uuid_repr: Option<UuidRepresentation>) -> Result<Vec<u8>>;
    fn to_guid(self, uuid_repr: Option<UuidRepresentation>) -> Result<Vec<u8>>;
    fn to_f64(&self) -> Result<(f64, Option<ODBCError>)>;
    fn to_f32(&self) -> Result<(f32, Option<ODBCError>)>;
    fn to_i64(&self) -> Result<(i64, Option<ODBCError>)>;
    fn to_i32(&self) -> Result<(i32, Option<ODBCError>)>;
    fn to_u64(&self) -> Result<(u64, Option<ODBCError>)>;
    fn to_u32(&self) -> Result<(u32, Option<ODBCError>)>;
    fn to_bit(&self) -> Result<(u8, Option<ODBCError>)>;
    fn to_datetime(&self) -> Result<(DateTime<Utc>, Option<ODBCError>)>;
    fn to_date(&self) -> Result<(NaiveDate, Option<ODBCError>)>;
    fn to_time(&self) -> Result<(NaiveTime, Option<ODBCError>)>;
    fn to_type_str(&self) -> &'static str;
}

fn f64_to_bit(f: f64) -> Result<(u8, Option<ODBCError>)> {
    if f == 0.0 {
        Ok((0u8, None))
    } else if f == 1.0 {
        Ok((1u8, None))
    } else if f > 0.0 && f < 1.0 {
        Ok((0u8, Some(ODBCError::FractionalTruncation(f.to_string()))))
    } else if f > 1.0 && f < 2.0 {
        Ok((1u8, Some(ODBCError::FractionalTruncation(f.to_string()))))
    } else {
        Err(ODBCError::IntegralTruncation(f.to_string()))
    }
}

fn i64_to_bit(i: i64) -> Result<(u8, Option<ODBCError>)> {
    match i {
        0 => Ok((0u8, None)),
        1 => Ok((1u8, None)),
        _ => Err(ODBCError::IntegralTruncation(i.to_string())),
    }
}

fn string_contains_fractional_precision_micros(s: &str) -> bool {
    // pull out microseconds by splitting on decimal place (only one in valid time[stamp])
    // use regex to separate fractional seconds from any timezone formatting
    Regex::new(r"(\.\d+)")
        .unwrap()
        .find(s)
        .map_or(false, |mat| mat.as_str().len() > 10) // 9 digits plus a '.'
}

fn from_string(s: &str, conversion_error_type: &'static str) -> Result<f64> {
    f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(conversion_error_type))
}

impl IntoCData for Bson {
    fn to_json_val(self, uuid_repr: Option<UuidRepresentation>) -> Value {
        match self {
            Bson::Array(v) => {
                Value::Array(v.into_iter().map(|b| b.to_json_val(uuid_repr)).collect())
            }
            Bson::Document(v) => Value::Object(
                v.into_iter()
                    .map(|(k, v)| (k, v.to_json_val(uuid_repr)))
                    .collect(),
            ),
            Bson::String(s) => Value::String(s),
            Bson::Binary(b) if b.subtype == BinarySubtype::Uuid => {
                json!({"$uuid": b.to_uuid().unwrap().to_string()})
            }
            Bson::Binary(b) if b.subtype == BinarySubtype::UuidOld => {
                json!({"$uuid": b.to_uuid_with_representation(uuid_repr.unwrap_or(UuidRepresentation::PythonLegacy)).unwrap().to_string()})
            }
            _ => self.into_relaxed_extjson(),
        }
    }
    fn to_json(self, uuid_repr: Option<UuidRepresentation>) -> String {
        match self {
            Bson::String(s) => s,
            _ => self.to_json_val(uuid_repr).to_string(),
        }
    }

    fn to_binary(self, uuid_repr: Option<UuidRepresentation>) -> Result<Vec<u8>> {
        Ok(self.to_json(uuid_repr).into_bytes())
    }

    fn to_guid(self, uuid_repr: Option<UuidRepresentation>) -> Result<Vec<u8>> {
        match self {
            Bson::Binary(b) if b.subtype != BinarySubtype::Uuid => Err(
                ODBCError::RestrictedDataType("binary with non-uuid subtype", GUID),
            ),
            Bson::Binary(_) => Ok(self.to_json(uuid_repr).into_bytes()),
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), GUID)),
        }
    }

    fn to_f64(&self) -> Result<(f64, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => Ok((*f, None)),
            Bson::String(s) => Bson::Double(from_string(s, DOUBLE)?).to_f64(),
            Bson::Boolean(b) => Ok((if *b { 1.0 } else { 0.0 }, None)),
            Bson::Int32(i) => Ok((*i as f64, None)),
            Bson::Int64(i) => Ok((*i as f64, None)),
            Bson::Decimal128(d) => Bson::Double(from_string(&d.to_string(), DOUBLE)?).to_f64(),
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DOUBLE)),
        }
    }

    fn to_f32(&self) -> Result<(f32, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => {
                if *f > f32::MAX as f64 || *f < f32::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(f.to_string()))
                } else {
                    Ok((*f as f32, None))
                }
            }
            Bson::String(s) => Bson::Double(from_string(s, DOUBLE)?).to_f32(),

            Bson::Boolean(b) => Ok((if *b { 1.0 } else { 0.0 }, None)),
            Bson::Int32(i) => Ok((*i as f32, None)),
            Bson::Int64(i) => Ok((*i as f32, None)),
            Bson::Decimal128(d) => Bson::Double(from_string(&d.to_string(), DOUBLE)?).to_f32(),
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DOUBLE)),
        }
    }

    fn to_i64(&self) -> Result<(i64, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => {
                if *f > i64::MAX as f64 || *f < i64::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(f.to_string()))
                } else {
                    let info = if f.fract() != 0f64 {
                        Some(ODBCError::FractionalTruncation(f.to_string()))
                    } else {
                        None
                    };
                    Ok((*f as i64, info))
                }
            }
            Bson::String(s) => {
                Bson::Double(f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(INT64))?)
                    .to_i64()
            }
            Bson::Boolean(b) => Ok((i64::from(*b), None)),
            Bson::Int32(i) => Ok((*i as i64, None)),
            Bson::Int64(i) => Ok((*i, None)),
            // Note that this isn't perfect because there are some 64bit integer values that are
            // not representable as doubles. There *could* be a specific value where we will get a
            // different result here than if we had a conversion from Decimal128 to i64 directly.
            // We should update this when the bson crate supports Decimal128 entirely.
            Bson::Decimal128(_) => {
                let (out, _) = self.to_f64()?;
                if out > i64::MAX as f64 || out < i64::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(out.to_string()))
                } else {
                    Ok((
                        out as i64,
                        if out.floor() != out {
                            Some(ODBCError::FractionalTruncation(out.to_string()))
                        } else {
                            None
                        },
                    ))
                }
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), INT64)),
        }
    }

    fn to_i32(&self) -> Result<(i32, Option<ODBCError>)> {
        match self {
            Bson::Double(f) if *f > i32::MAX as f64 || *f < i32::MIN as f64 => {
                Err(ODBCError::IntegralTruncation(f.to_string()))
            }
            Bson::Int64(i) if *i > i32::MAX as i64 || *i < i32::MIN as i64 => {
                Err(ODBCError::IntegralTruncation(i.to_string()))
            }
            Bson::Decimal128(_) => {
                let (out, _) = self.to_f64()?;
                if out > i32::MAX as f64 || out < i32::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(out.to_string()))
                } else {
                    Ok((
                        out as i32,
                        if out.floor() != out {
                            Some(ODBCError::FractionalTruncation(out.to_string()))
                        } else {
                            None
                        },
                    ))
                }
            }
            Bson::String(s) => {
                Bson::Double(f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(INT32))?)
                    .to_i32()
            }
            _ => self.to_i64().map_or_else(
                |e| {
                    Err(match e {
                        ODBCError::RestrictedDataType(s, _) => {
                            ODBCError::RestrictedDataType(s, INT32)
                        }
                        ODBCError::InvalidCharacterValue(_) => {
                            ODBCError::InvalidCharacterValue(INT32)
                        }
                        _ => e,
                    })
                },
                |(u, w)| Ok((u as i32, w)),
            ),
        }
    }

    fn to_u64(&self) -> Result<(u64, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => {
                if *f < 0f64 || *f > u64::MAX as f64 {
                    Err(ODBCError::IntegralTruncation(f.to_string()))
                } else {
                    Ok((
                        *f as u64,
                        if f.fract() != 0f64 {
                            Some(ODBCError::FractionalTruncation(f.to_string()))
                        } else {
                            None
                        },
                    ))
                }
            }
            Bson::String(s) => Bson::Double(
                f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(UINT64))?,
            )
            .to_u64(),
            Bson::Boolean(b) => Ok((u64::from(*b), None)),
            Bson::Int32(i) => {
                if *i < 0i32 {
                    Err(ODBCError::IntegralTruncation(i.to_string()))
                } else {
                    Ok((*i as u64, None))
                }
            }
            Bson::Int64(i) => {
                if *i < 0i64 {
                    Err(ODBCError::IntegralTruncation(i.to_string()))
                } else {
                    Ok((*i as u64, None))
                }
            }
            // Note that this isn't perfect because there are some 64bit integer values that are
            // not representable as doubles. There *could* be a specific value where we will get a
            // different result here than if we had a conversion from Decimal128 to i64 directly.
            // We should update this when the bson crate supports Decimal128 entirely.
            Bson::Decimal128(_) => {
                let (out, _) = self.to_f64()?;
                if out > u64::MAX as f64 || out < u64::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(out.to_string()))
                } else {
                    Ok((
                        out as u64,
                        if out.floor() != out {
                            Some(ODBCError::FractionalTruncation(out.to_string()))
                        } else {
                            None
                        },
                    ))
                }
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), UINT64)),
        }
    }

    fn to_u32(&self) -> Result<(u32, Option<ODBCError>)> {
        match self {
            Bson::Double(f) if *f > u32::MAX as f64 || *f < 0f64 => {
                Err(ODBCError::IntegralTruncation(f.to_string()))
            }
            Bson::Int64(i) if *i > u32::MAX as i64 || *i < 0i64 => {
                Err(ODBCError::IntegralTruncation(i.to_string()))
            }
            Bson::Decimal128(_) => {
                let (out, _) = self.to_f64()?;
                if out > u32::MAX as f64 || out < u32::MIN as f64 {
                    Err(ODBCError::IntegralTruncation(out.to_string()))
                } else {
                    Ok((
                        out as u32,
                        if out.floor() != out {
                            Some(ODBCError::FractionalTruncation(out.to_string()))
                        } else {
                            None
                        },
                    ))
                }
            }
            Bson::Int32(i) if *i < 0i32 => Err(ODBCError::IntegralTruncation(i.to_string())),
            Bson::String(s) => Bson::Double(
                f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(UINT32))?,
            )
            .to_u32(),
            _ => self.to_i64().map_or_else(
                |e| {
                    Err(match e {
                        ODBCError::RestrictedDataType(s, _) => {
                            ODBCError::RestrictedDataType(s, UINT32)
                        }
                        ODBCError::InvalidCharacterValue(_) => {
                            ODBCError::InvalidCharacterValue(UINT32)
                        }
                        _ => e,
                    })
                },
                |(u, w)| Ok((u as u32, w)),
            ),
        }
    }

    fn to_bit(&self) -> Result<(u8, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => f64_to_bit(*f),
            Bson::String(s) => {
                Bson::Double(f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(BIT))?)
                    .to_bit()
            }
            Bson::Boolean(b) => Ok((u8::from(*b), None)),
            Bson::Int32(i) => i64_to_bit(*i as i64),
            Bson::Int64(i) => i64_to_bit(*i),
            Bson::Decimal128(_) => {
                let (f, _) = self.to_f64()?;
                f64_to_bit(f)
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), BIT)),
        }
    }

    fn to_datetime(&self) -> Result<(DateTime<Utc>, Option<ODBCError>)> {
        match self {
            Bson::DateTime(d) => Ok(((*d).into(), None)),
            Bson::String(s) => {
                // using '-' to check if input string contains a date and ':' to check if input string contains a time
                // parse (or set defaults) for date and time depending on contents of string
                let (date, time) = if s.contains('-') && s.contains(':') {
                    let dt = NaiveDateTime::parse_from_str(s, "%F %T%.f")
                        .or_else(|_| NaiveDateTime::parse_from_str(s, "%+"))
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?;
                    (dt.date(), dt.time())
                } else if s.contains('-') {
                    (
                        NaiveDate::parse_from_str(s, "%F")
                            .map_err(|_| ODBCError::InvalidDatetimeFormat)?,
                        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    )
                } else {
                    let time = NaiveTime::parse_from_str(s, "%T%.f")
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?;
                    (Utc::now().naive_utc().date(), time)
                };
                Ok((
                    TimeZone::from_utc_datetime(&Utc, &NaiveDateTime::new(date, time)),
                    string_contains_fractional_precision_micros(s)
                        .then_some(ODBCError::FractionalSecondsTruncation(s.clone())),
                ))
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DATETIME)),
        }
    }

    fn to_date(&self) -> Result<(NaiveDate, Option<ODBCError>)> {
        match self {
            Bson::DateTime(d) => {
                let chrono_datetime = (*d).to_chrono();
                Ok((
                    chrono_datetime.date_naive(),
                    (chrono_datetime.time() != NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap())
                        .then_some(ODBCError::TimeTruncation(d.to_string())),
                ))
            }
            Bson::String(s) => {
                let dt = if s.contains(':') {
                    NaiveDateTime::parse_from_str(s, "%F %T%.f")
                        .or_else(|_| NaiveDateTime::parse_from_str(s, "%+"))
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?
                } else {
                    NaiveDate::parse_from_str(s, "%F")
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                };
                Ok((
                    dt.date(),
                    (dt.time() != NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap())
                        .then_some(ODBCError::TimeTruncation(s.clone())),
                ))
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DATETIME)),
        }
    }

    fn to_time(&self) -> Result<(NaiveTime, Option<ODBCError>)> {
        match self {
            Bson::DateTime(d) => {
                let dt_chrono = (*d).to_chrono();
                Ok((
                    dt_chrono.time().with_nanosecond(0).unwrap(),
                    (dt_chrono.nanosecond() > 0)
                        .then_some(ODBCError::SecondsTruncation(d.to_string())),
                ))
            }
            Bson::String(s) => {
                let time = if s.contains('-') {
                    NaiveDateTime::parse_from_str(s, "%F %T%.f")
                        .or_else(|_| NaiveDateTime::parse_from_str(s, "%+"))
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?
                        .time()
                } else {
                    NaiveTime::parse_from_str(s, "%T%.f")
                        .map_err(|_| ODBCError::InvalidDatetimeFormat)?
                };

                // time strings with fractional seconds are invalid; timestamps with fractional seconds yield truncation
                if time.nanosecond() > 0 && !s.contains('-') {
                    Err(ODBCError::InvalidDatetimeFormat)
                } else {
                    Ok((
                        time.with_nanosecond(0).unwrap(),
                        (time.nanosecond() > 0).then_some(ODBCError::SecondsTruncation(s.clone())),
                    ))
                }
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DATETIME)),
        }
    }

    fn to_type_str(&self) -> &'static str {
        match self {
            Bson::Double(_) => "double",
            Bson::String(_) => "string",
            Bson::Document(_) => "object",
            Bson::Array(_) => "array",
            Bson::Binary(_) => "binData",
            Bson::Undefined => "undefined",
            Bson::ObjectId(_) => "objectId",
            Bson::Boolean(_) => "bool",
            Bson::DateTime(_) => "date",
            Bson::Null => "null",
            Bson::RegularExpression(_) => "regex",
            Bson::DbPointer(_) => "dbPointer",
            Bson::JavaScriptCode(_) => "javascript",
            Bson::JavaScriptCodeWithScope(_) => "javascriptWithScope",
            Bson::Symbol(_) => "symbol",
            Bson::Int32(_) => "int",
            Bson::Timestamp(_) => "timestamp",
            Bson::Int64(_) => "long",
            Bson::Decimal128(_) => "decimal",
            Bson::MinKey => "minKey",
            Bson::MaxKey => "maxKey",
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn format_binary(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    index: usize,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Vec<u8>,
    function_name: &str,
) -> SqlReturn {
    let sql_return = {
        let stmt = (*mongo_handle).as_statement().unwrap();
        isize_len::set_output_binary(
            stmt,
            data,
            col_num,
            index,
            target_value_ptr as *mut _,
            buffer_len as usize,
            str_len_or_ind_ptr,
        )
    };
    if sql_return == SqlReturn::SUCCESS_WITH_INFO {
        add_diag_with_function!(
            mongo_handle,
            ODBCError::OutStringTruncated(buffer_len as usize),
            function_name
        );
    }
    sql_return
}

macro_rules! char_data {
    ($mongo_handle:expr, $col_num:expr, $index:expr, $target_value_ptr:expr, $buffer_len:expr, $str_len_or_ind_ptr:expr, $data:expr, $func:path, $function_name:expr) => {{
        // force expressions used more than once.
        let (mongo_handle, buffer_len) = ($mongo_handle, $buffer_len);
        let sql_return = {
            let stmt = (*mongo_handle).as_statement().unwrap();
            $func(
                stmt,
                $data,
                $col_num,
                $index,
                $target_value_ptr as *mut _,
                $buffer_len as usize,
                $str_len_or_ind_ptr,
            )
        };
        if sql_return == SqlReturn::SUCCESS_WITH_INFO {
            add_diag_with_function!(
                mongo_handle,
                ODBCError::OutStringTruncated(buffer_len as usize),
                $function_name
            );
        }
        sql_return
    }};
}

macro_rules! fixed_data_with_warnings {
    ($mongo_handle:expr, $col_num:expr, $data:expr, $target_value_ptr:expr, $str_len_or_ind_ptr:expr) => {{
        let stmt = (*$mongo_handle).as_statement().unwrap();
        stmt.insert_var_data_cache($col_num, CachedData::Fixed);
        match $data {
            Ok((u, warning)) => {
                let sqlreturn =
                    isize_len::set_output_fixed_data(&u, $target_value_ptr, $str_len_or_ind_ptr);
                if let Some(warning) = warning {
                    stmt.errors.write().unwrap().push(warning);
                    return SqlReturn::SUCCESS_WITH_INFO;
                }
                sqlreturn
            }
            Err(e) => {
                stmt.errors.write().unwrap().push(e);
                SqlReturn::ERROR
            }
        }
    }};
}

pub unsafe fn format_datetime(
    mongo_handle: &MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    stmt.insert_var_data_cache(col_num, CachedData::Fixed);
    let dt = data.to_datetime();
    match dt {
        Ok((dt, warning)) => {
            let data = Timestamp {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
                hour: dt.hour() as u16,
                minute: dt.minute() as u16,
                second: dt.second() as u16,
                fraction: dt.nanosecond(),
            };
            let sqlreturn =
                isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr);
            if let Some(warning) = warning {
                stmt.errors.write().unwrap().push(warning);
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            sqlreturn
        }
        Err(e) => {
            stmt.errors.write().unwrap().push(e);
            SqlReturn::ERROR
        }
    }
}

pub unsafe fn format_time(
    mongo_handle: &MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    let time = data.to_time();
    stmt.insert_var_data_cache(col_num, CachedData::Fixed);
    match time {
        Ok((time, warning)) => {
            let data = Time {
                hour: time.hour() as u16,
                minute: time.minute() as u16,
                second: time.second() as u16,
            };
            let sqlreturn =
                isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr);
            if let Some(warning) = warning {
                stmt.errors.write().unwrap().push(warning);
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            sqlreturn
        }
        Err(e) => {
            stmt.errors.write().unwrap().push(e);
            SqlReturn::ERROR
        }
    }
}

pub unsafe fn format_date(
    mongo_handle: &MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    let dt = data.to_date();
    stmt.insert_var_data_cache(col_num, CachedData::Fixed);
    match dt {
        Ok((dt, warning)) => {
            let data = Date {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
            };
            let sqlreturn =
                isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr);
            if let Some(warning) = warning {
                stmt.errors.write().unwrap().push(warning);
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            sqlreturn
        }
        Err(e) => {
            stmt.errors.write().unwrap().push(e);
            SqlReturn::ERROR
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn format_cached_data(
    mongo_handle: &mut MongoHandle,
    cached_data: CachedData,
    col_or_param_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    function_name: &str,
) -> SqlReturn {
    match cached_data {
        // Fixed cannot be streamed, and this data has already been retrived before.
        a @ CachedData::Fixed => {
            let stmt = (*mongo_handle).as_statement().unwrap();
            // we need to insert Fixed so that we can return SqlReturn::NO_DATA if this is
            // called again.
            stmt.insert_var_data_cache(col_or_param_num, a);
            SqlReturn::NO_DATA
        }
        CachedData::Char(index, data) => {
            if target_type != CDataType::SQL_C_CHAR {
                let stmt = (*mongo_handle).as_statement().unwrap();
                stmt.insert_var_data_cache(col_or_param_num, CachedData::Char(index, data));
                return SqlReturn::NO_DATA;
            }
            char_data!(
                mongo_handle,
                col_or_param_num,
                index,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_string,
                function_name
            )
        }
        CachedData::WChar(index, data) => {
            if target_type != CDataType::SQL_C_WCHAR {
                let stmt = (*mongo_handle).as_statement().unwrap();

                stmt.insert_var_data_cache(col_or_param_num, CachedData::WChar(index, data));
                return SqlReturn::NO_DATA;
            }
            char_data!(
                mongo_handle,
                col_or_param_num,
                index,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_wstring_as_bytes,
                function_name
            )
        }
        CachedData::Bin(index, data) => {
            if target_type != CDataType::SQL_C_BINARY {
                let stmt = (*mongo_handle).as_statement().unwrap();

                stmt.insert_var_data_cache(col_or_param_num, CachedData::Bin(index, data));
                return SqlReturn::NO_DATA;
            }
            crate::api::data::format_binary(
                mongo_handle,
                col_or_param_num,
                index,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                function_name,
            )
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn format_bson_data(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
    function_name: &str,
) -> SqlReturn {
    // If the data is null or undefined we immediately return NULL_DATA indicator.
    match data {
        Bson::Null | Bson::Undefined => {
            let stmt = (*mongo_handle).as_statement().unwrap();

            if str_len_or_ind_ptr.is_null() {
                stmt.errors
                    .write()
                    .unwrap()
                    .push(ODBCError::IndicatorVariableRequiredButNotSupplied);
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            *str_len_or_ind_ptr = definitions::SQL_NULL_DATA;
            stmt.insert_var_data_cache(col_num, CachedData::Fixed);
            return SqlReturn::SUCCESS;
        }
        _ => {}
    }

    let uuid_repr = match (*mongo_handle).as_statement_connection() {
        Some(conn) => match conn.mongo_connection.read() {
            Ok(conn) => {
                if conn.as_ref().is_some() {
                    conn.as_ref().unwrap().uuid_repr
                } else {
                    None
                }
            }
            Err(_) => None,
        },
        None => None,
    };

    match target_type {
        CDataType::SQL_C_BINARY | CDataType::SQL_C_GUID => {
            let data = if target_type == CDataType::SQL_C_GUID {
                data.to_guid(uuid_repr)
            } else {
                data.to_binary(uuid_repr)
            };
            match data {
                Ok(data) => format_binary(
                    mongo_handle,
                    col_num,
                    0usize,
                    target_value_ptr,
                    buffer_len,
                    str_len_or_ind_ptr,
                    data,
                    function_name,
                ),
                Err(e) => {
                    let stmt = (*mongo_handle).as_statement().unwrap();

                    stmt.errors.write().unwrap().push(e);
                    SqlReturn::ERROR
                }
            }
        }
        CDataType::SQL_C_CHAR => {
            let data = data.to_json(uuid_repr).bytes().collect::<Vec<u8>>();
            char_data!(
                mongo_handle,
                col_num,
                0usize,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_string,
                function_name
            )
        }
        CDataType::SQL_C_WCHAR => {
            let data = cstr::to_widechar_vec(&data.to_json(uuid_repr));
            char_data!(
                mongo_handle,
                col_num,
                0usize,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_wstring_as_bytes,
                function_name
            )
        }
        CDataType::SQL_C_BIT => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_bit(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_DOUBLE => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_f64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_FLOAT => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_f32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_SBIGINT => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_i64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_UBIGINT => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_u64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_SLONG => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_i32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_ULONG => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_u32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SQL_C_TIMESTAMP | CDataType::SQL_C_TYPE_TIMESTAMP => format_datetime(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        CDataType::SQL_C_TIME | CDataType::SQL_C_TYPE_TIME => format_time(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        CDataType::SQL_C_DATE | CDataType::SQL_C_TYPE_DATE => format_date(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        other => {
            add_diag_with_function!(
                mongo_handle,
                ODBCError::UnimplementedDataType(format!("{other:?}")),
                function_name
            );
            SqlReturn::ERROR
        }
    }
}

///
/// set_output_wstring_helper writes [`message`] to the *WideChar [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
unsafe fn set_output_wstring_helper(
    message: &[WideChar],
    output_ptr: *mut WideChar,
    buffer_len: usize,
) -> (usize, SqlReturn) {
    // If the output_ptr is null or no buffer space has been allocated, we need
    // to return SUCCESS_WITH_INFO.
    if output_ptr.is_null() || buffer_len == 0 {
        return (0usize, SqlReturn::SUCCESS_WITH_INFO);
    }
    // TODO SQL-1084: This will currently not work when we need to truncate data that takes more than
    // two bytes, such as emojis because it's assuming every character is 2 bytes.
    // Actually, this is not clear now. The spec suggests it may be up to the user to correctly
    // reassemble parts.
    let num_chars_written = write_wstring_slice_to_buffer(message, buffer_len, output_ptr) as usize;
    // return the number of characters in the message string, excluding the
    // null terminator
    if num_chars_written <= message.len() {
        (num_chars_written - 1, SqlReturn::SUCCESS_WITH_INFO)
    } else {
        (message.len(), SqlReturn::SUCCESS)
    }
}

///
/// set_output_string_helper writes [`message`] to the *Char [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
unsafe fn set_output_string_helper(
    message: &[u8],
    output_ptr: *mut Char,
    buffer_len: usize,
) -> (usize, SqlReturn) {
    // If the output_ptr is null or no buffer space has been allocated, we need
    // to return SUCCESS_WITH_INFO.
    if output_ptr.is_null() || buffer_len == 0 {
        return (0usize, SqlReturn::SUCCESS_WITH_INFO);
    }

    let num_chars_written = write_string_slice_to_buffer(message, buffer_len, output_ptr) as usize;

    // return the number of characters in the message string, excluding the
    // null terminator
    if num_chars_written <= message.len() {
        (num_chars_written - 1, SqlReturn::SUCCESS_WITH_INFO)
    } else {
        (message.len(), SqlReturn::SUCCESS)
    }
}

///
/// set_output_binary_helper writes [`message`] to the *Char [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
unsafe fn set_output_binary_helper(
    data: &[u8],
    output_ptr: *mut Char,
    buffer_len: usize,
) -> (usize, SqlReturn) {
    // If the output_ptr is null or no buffer space has been allocated, we need
    // to return SUCCESS_WITH_INFO.
    if output_ptr.is_null() || buffer_len == 0 {
        return (0usize, SqlReturn::SUCCESS_WITH_INFO);
    }

    let num_bytes_written = write_binary_slice_to_buffer(data, buffer_len, output_ptr) as usize;

    // return the number of characters in the binary
    if num_bytes_written < data.len() {
        (num_bytes_written, SqlReturn::SUCCESS_WITH_INFO)
    } else {
        (num_bytes_written, SqlReturn::SUCCESS)
    }
}

pub mod i16_len {
    use super::*;
    ///
    /// set_output_wstring_as_bytes writes [`message`] to the Pointer [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of *BYTES* written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring_as_bytes(
        message: &str,
        output_ptr: Pointer,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        let message = cstr::to_widechar_vec(message);
        let (len, ret) = set_output_wstring_helper(
            &message,
            output_ptr as *mut WideChar,
            buffer_len / size_of::<WideChar>(),
        );
        // Only copy the length if the pointer is not null
        ptr_safe_write(text_length_ptr, (size_of::<WideChar>() * len) as SmallInt);
        ret
    }

    ///
    /// set_output_wstring writes [`message`] to the *WideChar [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring(
        message: &str,
        output_ptr: *mut WideChar,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        let message = cstr::to_widechar_vec(message);
        let (len, ret) = set_output_wstring_helper(&message, output_ptr, buffer_len);
        // Only copy the length if the pointer is not null
        ptr_safe_write(text_length_ptr, len as SmallInt);
        ret
    }

    ///
    /// set_output_fixed_data writes [`data`], which must be a fixed sized type, to the Pointer [`output_ptr`].
    /// ODBC drivers assume the output buffer is large enough for fixed types, and are allowed to
    /// overwrite the buffer if too small a buffer is passed.
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_fixed_data<T: core::fmt::Debug>(
        data: &T,
        output_ptr: Pointer,
        data_len_ptr: *mut SmallInt,
    ) -> SqlReturn {
        // If the output_ptr is NULL, we should still return the length of the message.
        ptr_safe_write(data_len_ptr, size_of::<T>() as i16);

        if output_ptr.is_null() {
            return SqlReturn::SUCCESS_WITH_INFO;
        }
        write_fixed_data(data, output_ptr);
        SqlReturn::SUCCESS
    }
}

pub mod i32_len {
    use super::*;
    ///
    /// set_output_wstring_as_bytes writes [`message`] to the Pointer [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in *BYTES*; the message should be truncated
    /// if it is longer than the buffer length. The number of *BYTES* written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring_as_bytes(
        message: &str,
        output_ptr: Pointer,
        buffer_len: usize,
        text_length_ptr: *mut Integer,
    ) -> SqlReturn {
        let (len, ret) = set_output_wstring_helper(
            &cstr::to_widechar_vec(message),
            output_ptr as *mut WideChar,
            buffer_len / size_of::<WideChar>(),
        );

        ptr_safe_write(text_length_ptr, (size_of::<WideChar>() * len) as Integer);
        ret
    }

    ///
    /// set_output_fixed_data writes [`data`], which must be a fixed sized type, to the Pointer [`output_ptr`].
    /// ODBC drivers assume the output buffer is large enough for fixed types, and are allowed to
    /// overwrite the buffer if too small a buffer is passed.
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_fixed_data<T: core::fmt::Debug>(
        data: &T,
        output_ptr: Pointer,
        data_len_ptr: *mut Integer,
    ) -> SqlReturn {
        // If the output_ptr is NULL, we should still return the length of the message.
        ptr_safe_write(data_len_ptr, size_of::<T>() as i32);

        if output_ptr.is_null() {
            return SqlReturn::SUCCESS_WITH_INFO;
        }
        write_fixed_data(data, output_ptr);
        SqlReturn::SUCCESS
    }
}

pub mod isize_len {
    use super::*;
    ///
    /// set_output_wstring writes [`message`] to the Pointer [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of *BYTES* written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring_as_bytes(
        stmt: &Statement,
        message: Vec<WideChar>,
        col_num: USmallInt,
        index: usize,
        output_ptr: *mut WideChar,
        buffer_len: usize,
        text_length_ptr: *mut Len,
    ) -> SqlReturn {
        // This should be impossible per the DM.
        if output_ptr.is_null() {
            return SqlReturn::ERROR;
        }
        // TODO Power BI: This will return NO_DATA if the string is size 0 to begin with, not just
        // when the data runs out. Check to see if this is correct behavior.
        if index >= message.len() {
            ptr_safe_write(text_length_ptr, 0);
            return SqlReturn::NO_DATA;
        }
        let (len, ret) = set_output_wstring_helper(
            message.get(index..).unwrap(),
            output_ptr,
            buffer_len / size_of::<WideChar>(),
        );
        // the returned length should always be the total length of the data.
        ptr_safe_write(
            text_length_ptr,
            (size_of::<WideChar>() * (message.len() - index)) as Len,
        );
        stmt.insert_var_data_cache(col_num, CachedData::WChar(index + len, message));
        ret
    }

    ///
    /// set_output_string writes [`message`] to the *Char [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_string(
        stmt: &Statement,
        message: Vec<u8>,
        col_num: USmallInt,
        index: usize,
        output_ptr: *mut Char,
        buffer_len: usize,
        text_length_ptr: *mut Len,
    ) -> SqlReturn {
        // This should be impossible per the DM.
        if output_ptr.is_null() {
            return SqlReturn::ERROR;
        }
        // TODO Power BI: This will return NO_DATA if the string is size 0 to begin with, not just
        // when the data runs out. Check to see if this is correct behavior.
        if index >= message.len() {
            ptr_safe_write(text_length_ptr, 0);
            return SqlReturn::NO_DATA;
        }
        let (len, ret) =
            set_output_string_helper(message.get(index..).unwrap(), output_ptr, buffer_len);
        // the returned length should always be the total length of the data.
        ptr_safe_write(text_length_ptr, (message.len() - index) as Len);
        // The length parameter does not matter because character data uses 8bit words and
        // we can obtain it from message.chars().count() above.
        stmt.insert_var_data_cache(col_num, CachedData::Char(len + index, message));
        ret
    }

    ///
    /// set_output_binary writes [`message`] to the *Char [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_binary(
        stmt: &Statement,
        data: Vec<u8>,
        col_num: USmallInt,
        index: usize,
        output_ptr: *mut Char,
        buffer_len: usize,
        text_length_ptr: *mut Len,
    ) -> SqlReturn {
        // This should be impossible per the DM.
        if output_ptr.is_null() {
            return SqlReturn::ERROR;
        }
        // TODO Power BI: This will return NO_DATA if the data is size 0 to begin with, not just
        // when the data runs out. Check to see if this is correct behavior.
        if index >= data.len() {
            ptr_safe_write(text_length_ptr, 0);
            return SqlReturn::NO_DATA;
        }
        let (len, ret) =
            set_output_binary_helper(data.get(index..).unwrap(), output_ptr, buffer_len);
        ptr_safe_write(text_length_ptr, (data.len() - index) as Len);
        stmt.insert_var_data_cache(col_num, CachedData::Bin(len + index, data));
        ret
    }

    ///
    /// set_output_fixed_data writes [`data`], which must be a fixed sized type, to the Pointer [`output_ptr`].
    /// ODBC drivers assume the output buffer is large enough for fixed types, and are allowed to
    /// overwrite the buffer if too small a buffer is passed.
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_fixed_data<T: core::fmt::Debug>(
        data: &T,
        output_ptr: Pointer,
        data_len_ptr: *mut Len,
    ) -> SqlReturn {
        // This should be impossible per the DM.
        if output_ptr.is_null() {
            return SqlReturn::ERROR;
        }

        // If the output_ptr is NULL, we should still return the length of the message.
        ptr_safe_write(data_len_ptr, size_of::<T>() as isize);

        write_fixed_data(data, output_ptr);
        SqlReturn::SUCCESS
    }
}

///
/// ptr_safe_write writes the given data to [`ptr`].
///
/// # Safety
/// This writes to a raw C-pointers
///
pub unsafe fn ptr_safe_write<T>(ptr: *mut T, data: T) {
    if !ptr.is_null() {
        *ptr = data;
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use bson::bson;
    use chrono::offset::TimeZone;
    #[test]
    fn date_format() {
        assert_eq!(
            Utc.timestamp_opt(1003483404, 123000000).unwrap(),
            bson!("2001-10-19T09:23:24.123Z").to_datetime().unwrap().0
        );
    }

    mod conversion {
        use std::{collections::HashMap, f64::consts::PI};

        use crate::{api::data::IntoCData, map};
        use bson::Bson;
        use constants::{
            OdbcState, FRACTIONAL_TRUNCATION, INTEGRAL_TRUNCATION, INVALID_CHARACTER_VALUE,
            INVALID_DATETIME_FORMAT,
        };

        use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

        macro_rules! test_conversion_ok {
            (input = $input:expr, method = $method:tt, expected = $expected:expr, info = $info:expr) => {
                let actual = $input.$method();
                match actual {
                    Ok(r) => {
                        assert_eq!(
                            $expected,
                            r.0,
                            "expected {:?}, got {:?} calling method {} with {}",
                            $expected,
                            r.0,
                            stringify!($method),
                            $input
                        );
                        if $info.is_some() {
                            let info = $info.unwrap();
                            assert_eq!(
                                info, r.1.clone().unwrap().get_sql_state(),
                                "expected success with info {:?}, got {:?} calling method {} with {}",
                                info, r.1.unwrap().get_sql_state(), stringify!($method), $input
                            );
                        }
                    }
                    Err(e) => unreachable!("should not have had an err {:?} calling method {} with {}", e, stringify!($method), $input),
                }
            };
        }
        macro_rules! test_conversion_err {
            (input = $input:expr, method = $method:tt, expected = $expected:expr, info = $info:expr) => {
                let actual = $input.$method();
                match actual {
                    Err(e) => {
                        let info = $info.unwrap();
                        assert_eq!(
                            info,
                            e.get_sql_state(),
                            "expected {:?}, got {:?} calling method {:?} on {}",
                            info,
                            e.get_sql_state(),
                            stringify!($method),
                            $input
                        );
                    }
                    Ok(r) => unreachable!(
                        "should not have had an Ok {:?} calling method {} with {}",
                        r,
                        stringify!($method),
                        $input
                    ),
                }
            };
        }

        macro_rules! test_it {
            ($bson:expr,$v:expr) => {
                $v.iter().for_each(
                    |(method, expected, test, info)| match (method, test, info) {
                        (&"i64", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_i64,
                                expected = *expected as i64,
                                info = info
                            );
                        }
                        (&"i64", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_i64,
                                expected = *expected as i64,
                                info = info
                            );
                        }
                        (&"i32", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_i32,
                                expected = *expected as i32,
                                info = info
                            );
                        }
                        (&"i32", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_i32,
                                expected = *expected as i32,
                                info = info
                            );
                        }
                        (&"u64", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_u64,
                                expected = *expected as u64,
                                info = info
                            );
                        }
                        (&"u64", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_u64,
                                expected = *expected as u64,
                                info = info
                            );
                        }
                        (&"u32", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_u32,
                                expected = *expected as u32,
                                info = info
                            );
                        }
                        (&"u32", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_u32,
                                expected = *expected as u32,
                                info = info
                            );
                        }
                        (&"f64", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_f64,
                                expected = *expected as f64,
                                info = info
                            );
                        }
                        (&"f64", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_f64,
                                expected = *expected as f64,
                                info = info
                            );
                        }
                        (&"f32", Ok(()), info) => {
                            test_conversion_ok!(
                                input = $bson,
                                method = to_f32,
                                expected = *expected as f32,
                                info = info
                            );
                        }
                        (&"f32", Err(()), info) => {
                            test_conversion_err!(
                                input = $bson,
                                method = to_f32,
                                expected = *expected as f32,
                                info = info
                            );
                        }
                        _ => unimplemented!(),
                    },
                );
            };
        }

        // This also tests doubles (f64) as well, since the implementation
        // for converting strings to numerics converts the string to
        // an f64
        #[test]
        fn string_conversions_to_numerics() {
            type V = Vec<(
                &'static str,
                i32,
                Result<(), ()>,
                Option<OdbcState<'static>>,
            )>;
            let strings: HashMap<String, V> = map! {
                (-PI).to_string() => vec![
                    ("i64", -3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                    ("i32", -3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                    ("u64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                ],
                PI.to_string() => vec![
                    ("i64", 3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                    ("i32", 3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                    ("u64", 3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                    ("u32", 3, Ok(()), Some(FRACTIONAL_TRUNCATION)),
                ],
                i128::MIN.to_string() => vec![
                    ("i64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("i32", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                ],
                i128::MAX.to_string() => vec![
                    ("i64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("i32", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                ],
                i32::MAX.to_string() => vec![
                    ("i64", i32::MAX, Ok(()), None),
                    ("i32", i32::MAX, Ok(()), None),
                    ("u64", i32::MAX, Ok(()), None),
                    ("u32", i32::MAX, Ok(()), None),
                ],
                "foo".to_string() => vec![
                    ("i64", 0, Err(()), Some(INVALID_CHARACTER_VALUE)),
                    ("i32", 0, Err(()), Some(INVALID_CHARACTER_VALUE)),
                    ("u64", 0, Err(()), Some(INVALID_CHARACTER_VALUE)),
                    ("u32", 0, Err(()), Some(INVALID_CHARACTER_VALUE)),
                ],
            };
            strings.iter().for_each(|(k, v)| {
                let bson = Bson::String(k.to_string());
                test_it!(bson, v);
            });
        }
        #[test]
        fn string_conversions_to_floats() {
            type V = Vec<(
                &'static str,
                f64,
                Result<(), ()>,
                Option<OdbcState<'static>>,
            )>;
            let strings: HashMap<String, V> = map! {
                (-PI).to_string() => vec![
                    ("f64", -PI, Ok(()), None),
                    ("f32", -PI, Ok(()), None),
                ],
                PI.to_string() => vec![
                    ("f64", PI, Ok(()), None),
                    ("f32", PI, Ok(()), None),
                ],
                f64::MAX.to_string() => vec![
                    ("f64", f64::MAX, Ok(()), None),
                    ("f32", 0., Err(()), Some(INTEGRAL_TRUNCATION)),
                ],
                "foo".to_string() => vec![
                    ("f64", 0., Err(()), Some(INVALID_CHARACTER_VALUE)),
                    ("f32", 0., Err(()), Some(INVALID_CHARACTER_VALUE)),
                ],
            };
            strings.iter().for_each(|(k, v)| {
                let bson = Bson::String(k.to_string());
                test_it!(bson, v);
            });
        }

        #[test]
        fn int64_conversions_to_numerics() {
            type V = Vec<(
                &'static str,
                i64,
                Result<(), ()>,
                Option<OdbcState<'static>>,
            )>;
            let int_64s: HashMap<i64, V> = map! {
                i64::MAX => vec![
                    ("i64", i64::MAX, Ok(()), None),
                    ("i32", 0i64, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u64", i64::MAX, Ok(()), None),
                    ("u32", 0i64, Err(()), Some(INTEGRAL_TRUNCATION))
                ],
                i32::MAX as i64 => vec![
                    ("i64", i32::MAX as i64, Ok(()), None),
                    ("i32", i32::MAX as i64, Ok(()), None),
                    ("u64", i32::MAX as i64, Ok(()), None),
                    ("u32", i32::MAX as i64, Ok(()), None)
                ],
                i64::MIN => vec![
                    ("i64", i64::MIN, Ok(()), None),
                    ("i32", 0i64, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u64", 0i64, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0i64, Err(()), Some(INTEGRAL_TRUNCATION))
                ],
                i32::MIN as i64 => vec![
                    ("i64", i32::MIN as i64, Ok(()), None),
                    ("i32", i32::MIN as i64, Ok(()), None),
                    ("u64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0, Err(()), Some(INTEGRAL_TRUNCATION))
                ],
            };

            int_64s.iter().for_each(|(k, v)| {
                let bson = Bson::Int64(*k);
                test_it!(bson, v);
            })
        }

        #[test]
        fn int_32_conversions_to_numerics() {
            type V = Vec<(
                &'static str,
                i32,
                Result<(), ()>,
                Option<OdbcState<'static>>,
            )>;
            let int_32s: HashMap<i32, V> = map! {
                i32::MAX => vec![
                    ("i64", i32::MAX, Ok(()), None),
                    ("i32", i32::MAX, Ok(()), None),
                    ("u64", i32::MAX, Ok(()), None),
                    ("u32", i32::MAX, Ok(()), None)
                ],
                i32::MIN => vec![
                    ("i64", i32::MIN, Ok(()), None),
                    ("i32", i32::MIN, Ok(()), None),
                    ("u64", 0, Err(()), Some(INTEGRAL_TRUNCATION)),
                    ("u32", 0, Err(()), Some(INTEGRAL_TRUNCATION))
                ]
            };

            int_32s.iter().for_each(|(k, v)| {
                let bson = Bson::Int32(*k);
                test_it!(bson, v);
            })
        }

        #[test]
        fn string_conversions_to_datetimes() {
            use chrono::TimeZone;
            let date_expectation: DateTime<Utc> = "2014-11-28T00:00:00Z".parse().unwrap();
            let datetime_expectation_no_millis: DateTime<Utc> =
                "2014-11-28T09:23:24Z".parse().unwrap();
            let datetime_expectation_millis: DateTime<Utc> =
                "2014-11-28T09:23:24.123456789Z".parse().unwrap();
            let today_plus_time_no_millis_expectation = TimeZone::from_utc_datetime(
                &Utc,
                &Utc::now().date_naive().and_hms_opt(9, 23, 24).unwrap(),
            );
            let today_plus_time_millis_expectation = TimeZone::from_utc_datetime(
                &Utc,
                &Utc::now()
                    .naive_utc()
                    .date()
                    .and_hms_nano_opt(9, 23, 24, 123456789)
                    .unwrap(),
            );

            type V = Vec<(
                &'static str,
                DateTime<Utc>,
                Result<(), ()>,
                Option<OdbcState<'static>>,
            )>;
            let test_cases: V = vec![
                (
                    "2014-11-28 09:23:24",
                    datetime_expectation_no_millis,
                    Ok(()),
                    None,
                ),
                (
                    "2014-11-28 09:23:24.123456789",
                    datetime_expectation_millis,
                    Ok(()),
                    None,
                ),
                (
                    "2014-11-28 09:23:24.1234567890",
                    datetime_expectation_millis,
                    Ok(()),
                    Some(FRACTIONAL_TRUNCATION),
                ),
                (
                    "2014-11-28T09:23:24.123456789Z",
                    datetime_expectation_millis,
                    Ok(()),
                    None,
                ),
                (
                    "2014-11-28T09:23:24.1234567890Z",
                    datetime_expectation_millis,
                    Ok(()),
                    Some(FRACTIONAL_TRUNCATION),
                ),
                ("2014-11-28", date_expectation, Ok(()), None),
                (
                    "11/28/2014",
                    datetime_expectation_no_millis,
                    Err(()),
                    Some(INVALID_DATETIME_FORMAT),
                ),
                (
                    "2014-35-80",
                    datetime_expectation_no_millis,
                    Err(()),
                    Some(INVALID_DATETIME_FORMAT),
                ),
                (
                    "2014-30-90 09:23:24.1234567890",
                    datetime_expectation_no_millis,
                    Err(()),
                    Some(INVALID_DATETIME_FORMAT),
                ),
                (
                    "30:23:24",
                    datetime_expectation_no_millis,
                    Err(()),
                    Some(INVALID_DATETIME_FORMAT),
                ),
                (
                    "09:23:24",
                    today_plus_time_no_millis_expectation,
                    Ok(()),
                    None,
                ),
                (
                    "09:23:24.123456789",
                    today_plus_time_millis_expectation,
                    Ok(()),
                    None,
                ),
                (
                    "09:23:24.1234567898",
                    today_plus_time_millis_expectation,
                    Ok(()),
                    Some(FRACTIONAL_TRUNCATION),
                ),
            ];
            test_cases
                .iter()
                .for_each(|(input, expectation, result, info)| {
                    match result {
                        Ok(_) => {
                            test_conversion_ok!(
                                input = Bson::String(input.to_string()),
                                method = to_datetime,
                                expected = *expectation,
                                info = info
                            );
                        }
                        Err(_) => {
                            test_conversion_err!(
                                input = Bson::String(input.to_string()),
                                method = to_datetime,
                                expected = None,
                                info = info
                            );
                        }
                    };
                });
        }

        #[test]
        fn string_conversions_to_dates() {
            let date_expectation = NaiveDate::from_ymd_opt(2014, 11, 28).unwrap();
            let test_cases: Vec<(&str, Result<(), ()>, Option<OdbcState<'static>>)> = vec![
                ("2014-11-28", Ok(()), None),
                ("11/28/2014", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("2014-22-22", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("10:15:30", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("2014-11-28 00:00:00", Ok(()), None),
                ("2014-11-28 00:00:00.000", Ok(()), None),
                ("2014-11-28T00:00:00Z", Ok(()), None),
                ("2014-11-28 10:15:30", Ok(()), Some(FRACTIONAL_TRUNCATION)),
            ];
            test_cases.iter().for_each(|(input, result, info)| {
                match result {
                    Ok(_) => {
                        test_conversion_ok!(
                            input = Bson::String(input.to_string()),
                            method = to_date,
                            expected = date_expectation,
                            info = info
                        );
                    }
                    Err(_) => {
                        test_conversion_err!(
                            input = Bson::String(input.to_string()),
                            method = to_date,
                            expected = None,
                            info = info
                        );
                    }
                };
            });
        }

        #[test]
        fn datetime_conversions_to_dates() {
            let date_expectation = NaiveDate::from_ymd_opt(2014, 11, 28).unwrap();
            let test_cases: [(chrono::DateTime<Utc>, Option<OdbcState<'static>>); 2] = [
                ("2014-11-28T00:00:00Z".parse().unwrap(), None),
                (
                    "2014-11-28T10:15:30.123Z".parse().unwrap(),
                    Some(FRACTIONAL_TRUNCATION),
                ),
            ];
            test_cases.iter().for_each(|(input, info)| {
                test_conversion_ok!(
                    input = Bson::DateTime(bson::DateTime::from_chrono(*input)),
                    method = to_date,
                    expected = date_expectation,
                    info = info
                );
            });
        }

        #[test]
        fn string_conversions_to_times() {
            let time_expectation = NaiveTime::from_hms_opt(10, 15, 30).unwrap();
            let test_cases: Vec<(&str, Result<(), ()>, Option<OdbcState<'static>>)> = vec![
                ("10:15:30", Ok(()), None),
                ("10:15:30.00000", Ok(()), None),
                ("10:15:30.123", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("25:15:30.123", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("2022-10-15", Err(()), Some(INVALID_DATETIME_FORMAT)),
                ("2014-11-28 10:15:30.000", Ok(()), None),
                (
                    "2014-11-28 10:15:30.1243",
                    Ok(()),
                    Some(FRACTIONAL_TRUNCATION),
                ),
            ];
            test_cases.iter().for_each(|(input, result, info)| {
                match result {
                    Ok(_) => {
                        test_conversion_ok!(
                            input = Bson::String(input.to_string()),
                            method = to_time,
                            expected = time_expectation,
                            info = info
                        );
                    }
                    Err(_) => {
                        test_conversion_err!(
                            input = Bson::String(input.to_string()),
                            method = to_time,
                            expected = None,
                            info = info
                        );
                    }
                };
            });
        }

        #[test]
        fn datetime_conversions_to_times() {
            let time_expectation = NaiveTime::from_hms_opt(10, 15, 30).unwrap();
            let test_cases: [(chrono::DateTime<Utc>, Option<OdbcState<'static>>); 2] = [
                ("2014-11-28T10:15:30Z".parse().unwrap(), None),
                (
                    "2014-11-28T10:15:30.123Z".parse().unwrap(),
                    Some(FRACTIONAL_TRUNCATION),
                ),
            ];
            test_cases.iter().for_each(|(input, info)| {
                test_conversion_ok!(
                    input = Bson::DateTime(bson::DateTime::from_chrono(*input)),
                    method = to_time,
                    expected = time_expectation,
                    info = info
                );
            });
        }
    }

    mod decimal128_to_f64 {
        use std::str::FromStr;

        use bson::{Bson, Decimal128};

        #[test]
        fn nan() {
            assert!(f64::from_str(
                &Bson::Decimal128(Decimal128::from_bytes([
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124
                ]))
                .to_string(),
            )
            .unwrap()
            .is_nan());
        }

        #[test]
        fn inf() {
            assert_eq!(
                f64::INFINITY,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_inf() {
            assert_eq!(
                f64::NEG_INFINITY,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn zero() {
            assert_eq!(
                0.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 48
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_zero() {
            assert_eq!(
                0.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 176
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn one() {
            // not sure why it drops .0 from 1 and not 0, but this is what the server does,
            // the algorithm is correct.
            assert_eq!(
                1.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 48
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_one() {
            // not sure why it drops .0 from -1 and not -0, but this is what the server does,
            // the algorithm is correct.
            assert_eq!(
                -1.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 176
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn big() {
            assert_eq!(
                412345123451234512345.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        217, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 48
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_big() {
            assert_eq!(
                -412345123451234512345.0,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        217, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 176
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn really_big() {
            assert_eq!(
                1.8E+305,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 50
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_really_big() {
            assert_eq!(
                -1.8E+305,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 178
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn really_small() {
            assert_eq!(
                1.8E-305,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 45
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_really_small() {
            assert_eq!(
                -1.8E-305,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 173
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn pi() {
            assert_eq!(
                std::f64::consts::PI,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 48
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_pi() {
            assert_eq!(
                -std::f64::consts::PI,
                f64::from_str(
                    &Bson::Decimal128(Decimal128::from_bytes([
                        96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 176
                    ]))
                    .to_string()
                )
                .unwrap(),
            );
        }
    }
}

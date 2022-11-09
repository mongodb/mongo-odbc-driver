use crate::{
    errors::ODBCError,
    handles::definitions::{CachedData, MongoHandle},
};
use bson::Bson;
use chrono::{
    offset::{TimeZone, Utc},
    DateTime, Datelike, Timelike,
};
use mongo_odbc_core::util::Decimal128Plus;
use odbc_sys::{CDataType, Date, Len, Pointer, Time, Timestamp, USmallInt};
use odbc_sys::{Char, Integer, SmallInt, SqlReturn, WChar};
use std::{cmp::min, collections::HashMap, mem::size_of, ptr::copy_nonoverlapping, str::FromStr};

const BINARY: &str = "Binary";
const DOUBLE: &str = "Double";
const INT32: &str = "Int32";
const INT64: &str = "Int64";
const UINT32: &str = "UInt32";
const UINT64: &str = "UInt64";
const BIT: &str = "Bit";
const DATETIME: &str = "DateTime";

type Result<T> = std::result::Result<T, ODBCError>;

/// IntoCData is just used for adding methods to bson::Bson.
trait IntoCData {
    fn to_json(self) -> String;
    fn to_binary(self) -> Result<Vec<u8>>;
    fn to_f64(&self) -> Result<f64>;
    fn to_f32(&self) -> Result<f32>;
    fn to_i64(&self) -> Result<(i64, Option<ODBCError>)>;
    fn to_i32(&self) -> Result<(i32, Option<ODBCError>)>;
    fn to_u64(&self) -> Result<(u64, Option<ODBCError>)>;
    fn to_u32(&self) -> Result<(u32, Option<ODBCError>)>;
    fn to_bit(&self) -> Result<(u8, Option<ODBCError>)>;
    fn to_datetime(&self) -> Result<DateTime<Utc>>;
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

impl IntoCData for Bson {
    fn to_json(self) -> String {
        match self {
            Bson::String(s) => s,
            // TODO SQL-1068 :we will have to test this manually because there is no way to create a Decimal128
            // in a unit test at this time. We could load a bson file, but since Decimal128 support
            // will be added soon, it's fine to wait on full support.
            Bson::Decimal128(d) => {
                format!("{{$numberDecimal: \"{}\"}}", d.to_formatted_string())
            }
            _ => self.into_canonical_extjson().to_string(),
        }
    }

    fn to_binary(self) -> Result<Vec<u8>> {
        match self {
            Bson::Binary(b) => Ok(b.bytes),
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), BINARY)),
        }
    }

    fn to_f64(&self) -> Result<f64> {
        match self {
            Bson::Double(f) => Ok(*f),
            Bson::String(s) => {
                f64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(s.clone(), DOUBLE))
            }
            Bson::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Bson::Int32(i) => Ok(*i as f64),
            Bson::Int64(i) => Ok(*i as f64),
            Bson::Decimal128(d) => {
                let d_str = d.to_formatted_string();
                f64::from_str(&d_str).map_err(|_| ODBCError::InvalidCharacterValue(d_str, DOUBLE))
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), DOUBLE)),
        }
    }

    fn to_f32(&self) -> Result<f32> {
        // this is mildly inefficient, unlike converting to i64 and then i32 which is essentially
        // free (on x86_64 you can literally just change the register address from e.g., RAX to EAX,
        // so it's literally a no-op), but since mongodb does not actually support 32 bit floats, we would need to do this
        // conversion somewhere, anyway.
        Ok(self.to_f64()? as f32)
    }

    fn to_i64(&self) -> Result<(i64, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => Ok((
                *f as i64,
                if f.floor() != *f {
                    Some(ODBCError::FractionalTruncation(f.to_string()))
                } else {
                    None
                },
            )),
            Bson::String(s) => Ok((
                i64::from_str(s).map_err(|_| ODBCError::InvalidCharacterValue(s.clone(), INT64))?,
                None,
            )),
            Bson::Boolean(b) => Ok((i64::from(*b), None)),
            Bson::Int32(i) => Ok((*i as i64, None)),
            Bson::Int64(i) => Ok((*i, None)),
            // Note that this isn't perfect because there are some 64bit integer values that are
            // not representable as doubles. There *could* be a specific value where we will get a
            // different result here than if we had a conversion from Decimal128 to i64 directly.
            // We should update this when the bson crate supports Decimal128 entirely.
            Bson::Decimal128(_) => {
                let out = self.to_f64()?;
                Ok((
                    out as i64,
                    if out.floor() != out {
                        Some(ODBCError::FractionalTruncation(out.to_string()))
                    } else {
                        None
                    },
                ))
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), INT64)),
        }
    }

    fn to_i32(&self) -> Result<(i32, Option<ODBCError>)> {
        match self {
            Bson::Double(x) if *x > i32::MAX as f64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            Bson::Int64(x) if *x > i32::MAX as i64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            Bson::Decimal128(x) if self.to_f64()? > i32::MAX as f64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            _ => self.to_i64().map_or_else(
                |e| match e {
                    ODBCError::RestrictedDataType(s, _) => {
                        Err(ODBCError::RestrictedDataType(s, INT32))
                    }
                    ODBCError::InvalidCharacterValue(s, _) => {
                        Err(ODBCError::InvalidCharacterValue(s, INT32))
                    }
                    _ => Err(e),
                },
                |(u, w)| Ok((u as i32, w)),
            ),
        }
    }

    fn to_u64(&self) -> Result<(u64, Option<ODBCError>)> {
        match self {
            Bson::Double(f) if *f < 0f64 => Err(ODBCError::IntegralTruncation(f.to_string())),
            Bson::Double(f) => Ok((
                *f as u64,
                if f.floor() != *f {
                    Some(ODBCError::FractionalTruncation(f.to_string()))
                } else {
                    None
                },
            )),
            Bson::String(s) => Ok((
                u64::from_str(s)
                    .map_err(|_| ODBCError::InvalidCharacterValue(s.clone(), UINT64))?,
                None,
            )),
            Bson::Boolean(b) => Ok((u64::from(*b), None)),
            Bson::Int32(i) if *i < 0i32 => Err(ODBCError::IntegralTruncation(i.to_string())),
            Bson::Int32(i) => Ok((*i as u64, None)),
            Bson::Int64(i) if *i < 0i64 => Err(ODBCError::IntegralTruncation(i.to_string())),
            Bson::Int64(i) => Ok((*i as u64, None)),
            // Note that this isn't perfect because there are some 64bit integer values that are
            // not representable as doubles. There *could* be a specific value where we will get a
            // different result here than if we had a conversion from Decimal128 to i64 directly.
            // We should update this when the bson crate supports Decimal128 entirely.
            Bson::Decimal128(_) => {
                let out = self.to_f64()?;
                if out < 0f64 {
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
            Bson::Double(x) if *x > u32::MAX as f64 || *x < 0f64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            Bson::Int64(x) if *x > u32::MAX as i64 || *x < 0i64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            Bson::Decimal128(x) if self.to_f64()? > u32::MAX as f64 || self.to_f64()? < 0f64 => {
                Err(ODBCError::IntegralTruncation(x.to_string()))
            }
            Bson::Int32(x) if *x < 0i32 => Err(ODBCError::IntegralTruncation(x.to_string())),
            _ => self.to_i64().map_or_else(
                |e| match e {
                    ODBCError::RestrictedDataType(s, _) => {
                        Err(ODBCError::RestrictedDataType(s, UINT32))
                    }
                    ODBCError::InvalidCharacterValue(s, _) => {
                        Err(ODBCError::InvalidCharacterValue(s, UINT32))
                    }
                    _ => Err(e),
                },
                |(u, w)| Ok((u as u32, w)),
            ),
        }
    }

    fn to_bit(&self) -> Result<(u8, Option<ODBCError>)> {
        match self {
            Bson::Double(f) => f64_to_bit(*f),
            Bson::String(s) => {
                let (i, warning) = self.to_i64().map_err(|e| {
                    if let ODBCError::InvalidCharacterValue(s, _) = e {
                        ODBCError::InvalidCharacterValue(s, BIT)
                    } else {
                        e
                    }
                })?;
                match i {
                    0 => Ok((0u8, warning)),
                    1 => Ok((1u8, warning)),
                    _ => Err(ODBCError::InvalidCharacterValue(s.clone(), BIT)),
                }
            }
            Bson::Boolean(b) => Ok((u8::from(*b), None)),
            Bson::Int32(i) => i64_to_bit(*i as i64),
            Bson::Int64(i) => i64_to_bit(*i),
            Bson::Decimal128(_) => {
                let f = self.to_f64()?;
                f64_to_bit(f)
            }
            o => Err(ODBCError::RestrictedDataType(o.to_type_str(), BIT)),
        }
    }

    fn to_datetime(&self) -> Result<DateTime<Utc>> {
        match self {
            Bson::DateTime(d) => Ok((*d).into()),
            Bson::String(s) => Utc
                .datetime_from_str(s, "%FT%H:%M:%S%.3fZ")
                .map_err(|_| ODBCError::InvalidDatetimeFormat(s.clone())),
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

pub unsafe fn format_binary(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    index: usize,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Vec<u8>,
) -> SqlReturn {
    let sql_return = {
        let stmt = (*mongo_handle).as_statement().unwrap();
        let mut guard = stmt.write().unwrap();
        isize_len::set_output_binary(
            data,
            col_num,
            index,
            guard.var_data_cache.as_mut().unwrap(),
            target_value_ptr as *mut _,
            buffer_len as usize,
            str_len_or_ind_ptr,
        )
    };
    if sql_return == SqlReturn::SUCCESS_WITH_INFO {
        mongo_handle.add_diag_info(ODBCError::OutStringTruncated(buffer_len as usize));
    }
    sql_return
}

macro_rules! char_data {
    ($mongo_handle:expr, $col_num:expr, $index:expr, $target_value_ptr:expr, $buffer_len:expr, $str_len_or_ind_ptr:expr, $data:expr, $func:path) => {{
        // force expressions used more than once.
        let (mongo_handle, buffer_len) = ($mongo_handle, $buffer_len);
        let sql_return = {
            let stmt = (*mongo_handle).as_statement().unwrap();
            let mut guard = stmt.write().unwrap();
            $func(
                $data,
                $col_num,
                $index,
                guard.var_data_cache.as_mut().unwrap(),
                $target_value_ptr as *mut _,
                $buffer_len as usize,
                $str_len_or_ind_ptr,
            )
        };
        if sql_return == SqlReturn::SUCCESS_WITH_INFO {
            mongo_handle.add_diag_info(ODBCError::OutStringTruncated(buffer_len as usize));
        }
        sql_return
    }};
}

macro_rules! fixed_data {
    ($mongo_handle:expr, $col_num:expr, $data:expr, $target_value_ptr:expr, $str_len_or_ind_ptr:expr) => {{
        let stmt = (*$mongo_handle).as_statement().unwrap();
        let mut guard = stmt.write().unwrap();
        let indices = guard.var_data_cache.as_mut().unwrap();
        indices.insert($col_num, CachedData::Fixed);
        match $data {
            Ok(f) => isize_len::set_output_fixed_data(&f, $target_value_ptr, $str_len_or_ind_ptr),
            Err(e) => {
                guard.errors.push(e);
                SqlReturn::ERROR
            }
        }
    }};
}

macro_rules! fixed_data_with_warnings {
    ($mongo_handle:expr, $col_num:expr, $data:expr, $target_value_ptr:expr, $str_len_or_ind_ptr:expr) => {{
        let stmt = (*$mongo_handle).as_statement().unwrap();
        let mut guard = stmt.write().unwrap();
        let indices = guard.var_data_cache.as_mut().unwrap();
        indices.insert($col_num, CachedData::Fixed);
        match $data {
            Ok((u, warning)) => {
                let sqlreturn =
                    isize_len::set_output_fixed_data(&u, $target_value_ptr, $str_len_or_ind_ptr);
                if let Some(warning) = warning {
                    guard.errors.push(warning);
                    return SqlReturn::SUCCESS_WITH_INFO;
                }
                sqlreturn
            }
            Err(e) => {
                guard.errors.push(e);
                SqlReturn::ERROR
            }
        }
    }};
}

pub unsafe fn format_datetime(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    let mut guard = stmt.write().unwrap();
    let indices = guard.var_data_cache.as_mut().unwrap();
    indices.insert(col_num, CachedData::Fixed);
    let dt = data.to_datetime();
    match dt {
        Ok(dt) => {
            let data = Timestamp {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
                hour: dt.hour() as u16,
                minute: dt.minute() as u16,
                second: dt.second() as u16,
                fraction: (dt.nanosecond() as f32 * 0.000001) as u32,
            };
            isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr)
        }
        Err(e) => {
            guard.errors.push(e);
            SqlReturn::ERROR
        }
    }
}

pub unsafe fn format_time(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    let mut guard = stmt.write().unwrap();
    let dt = data.to_datetime();
    let indices = guard.var_data_cache.as_mut().unwrap();
    indices.insert(col_num, CachedData::Fixed);
    match dt {
        Ok(dt) => {
            let data = Time {
                hour: dt.hour() as u16,
                minute: dt.minute() as u16,
                second: dt.second() as u16,
            };
            isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr)
        }
        Err(e) => {
            guard.errors.push(e);
            SqlReturn::ERROR
        }
    }
}

pub unsafe fn format_date(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    target_value_ptr: Pointer,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    let stmt = (*mongo_handle).as_statement().unwrap();
    let mut guard = stmt.write().unwrap();
    let indices = guard.var_data_cache.as_mut().unwrap();
    indices.insert(col_num, CachedData::Fixed);
    let dt = data.to_datetime();
    match dt {
        Ok(dt) => {
            let data = Date {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
            };
            isize_len::set_output_fixed_data(&data, target_value_ptr, str_len_or_ind_ptr)
        }
        Err(e) => {
            guard.errors.push(e);
            SqlReturn::ERROR
        }
    }
}

pub unsafe fn format_cached_data(
    mongo_handle: &mut MongoHandle,
    cached_data: CachedData,
    col_or_param_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    match cached_data {
        // Fixed cannot be streamed, and this data has already been retrived before.
        a @ CachedData::Fixed => {
            let stmt = (*mongo_handle).as_statement().unwrap();
            let mut guard = stmt.write().unwrap();
            let indices = guard.var_data_cache.as_mut().unwrap();
            // we need to insert Fixed so that we can return SqlReturn::NO_DATA if this is
            // called again.
            indices.insert(col_or_param_num, a);
            SqlReturn::NO_DATA
        }
        CachedData::Char(index, data) => {
            if target_type != CDataType::Char {
                let stmt = (*mongo_handle).as_statement().unwrap();
                let mut guard = stmt.write().unwrap();
                let indices = guard.var_data_cache.as_mut().unwrap();
                indices.insert(col_or_param_num, CachedData::Char(index, data));
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
                isize_len::set_output_string
            )
        }
        CachedData::WChar(index, data) => {
            if target_type != CDataType::WChar {
                let stmt = (*mongo_handle).as_statement().unwrap();
                let mut guard = stmt.write().unwrap();
                let indices = guard.var_data_cache.as_mut().unwrap();
                indices.insert(col_or_param_num, CachedData::WChar(index, data));
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
                isize_len::set_output_wstring
            )
        }
        CachedData::Bin(index, data) => {
            if target_type != CDataType::Binary {
                let stmt = (*mongo_handle).as_statement().unwrap();
                let mut guard = stmt.write().unwrap();
                let indices = guard.var_data_cache.as_mut().unwrap();
                indices.insert(col_or_param_num, CachedData::Bin(index, data));
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
            )
        }
    }
}

pub unsafe fn format_bson_data(
    mongo_handle: &mut MongoHandle,
    col_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    // If the data is null or undefined we immediately return NULL_DATA indicator.
    match data {
        Bson::Null | Bson::Undefined => {
            let stmt = (*mongo_handle).as_statement().unwrap();
            let mut guard = stmt.write().unwrap();
            let indices = guard.var_data_cache.as_mut().unwrap();
            if str_len_or_ind_ptr.is_null() {
                guard
                    .errors
                    .push(ODBCError::IndicatorVariableRequiredButNotSupplied);
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            *str_len_or_ind_ptr = odbc_sys::NULL_DATA;
            indices.insert(col_num, CachedData::Fixed);
            return SqlReturn::SUCCESS;
        }
        _ => {}
    }
    match target_type {
        CDataType::Binary => {
            let data = data.to_binary();
            match data {
                Ok(data) => format_binary(
                    mongo_handle,
                    col_num,
                    0usize,
                    target_value_ptr,
                    buffer_len,
                    str_len_or_ind_ptr,
                    data,
                ),
                Err(e) => {
                    let stmt = (*mongo_handle).as_statement().unwrap();
                    let mut guard = stmt.write().unwrap();
                    guard.errors.push(e);
                    SqlReturn::ERROR
                }
            }
        }
        CDataType::Char => {
            let data = data.to_json().bytes().collect::<Vec<u8>>();
            char_data!(
                mongo_handle,
                col_num,
                0usize,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_string
            )
        }
        CDataType::WChar => {
            let data = data.to_json().encode_utf16().collect::<Vec<u16>>();
            char_data!(
                mongo_handle,
                col_num,
                0usize,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
                data,
                isize_len::set_output_wstring
            )
        }
        CDataType::Bit => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_bit(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::Double => {
            fixed_data!(
                mongo_handle,
                col_num,
                data.to_f64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::Float => {
            fixed_data!(
                mongo_handle,
                col_num,
                data.to_f32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SBigInt | CDataType::Numeric => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_i64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::UBigInt => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_u64(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::SLong => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_i32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::ULong => {
            fixed_data_with_warnings!(
                mongo_handle,
                col_num,
                data.to_u32(),
                target_value_ptr,
                str_len_or_ind_ptr
            )
        }
        CDataType::TimeStamp | CDataType::TypeTimestamp => format_datetime(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        CDataType::Time | CDataType::TypeTime => format_time(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        CDataType::Date | CDataType::TypeDate => format_date(
            mongo_handle,
            col_num,
            target_value_ptr,
            str_len_or_ind_ptr,
            data,
        ),
        other => {
            mongo_handle.add_diag_info(ODBCError::UnimplementedDataType(format!("{:?}", other)));
            SqlReturn::ERROR
        }
    }
}

///
/// input_text_to_string converts an input cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_text_to_string(text: *const Char, len: usize) -> String {
    if (len as isize) < 0 {
        let mut dst = Vec::new();
        let mut itr = text;
        {
            while *itr != 0 {
                dst.push(*itr);
                itr = itr.offset(1);
            }
        }
        return String::from_utf8_unchecked(dst);
    }

    let mut dst = Vec::with_capacity(len);
    dst.set_len(len);
    copy_nonoverlapping(text, dst.as_mut_ptr(), len);
    String::from_utf8_unchecked(dst)
}

///
/// input_wtext_to_string converts an input cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_wtext_to_string(text: *const WChar, len: usize) -> String {
    if (len as isize) < 0 {
        let mut dst = Vec::new();
        let mut itr = text;
        {
            while *itr != 0 {
                dst.push(*itr);
                itr = itr.offset(1);
            }
        }
        return String::from_utf16_lossy(&dst);
    }

    let mut dst = Vec::with_capacity(len);
    dst.set_len(len);
    copy_nonoverlapping(text, dst.as_mut_ptr(), len);
    String::from_utf16_lossy(&dst)
}

///
/// set_sql_state writes the given sql state to the [`output_ptr`].
///
/// # Safety
/// This writes to a raw C-pointer
///
pub unsafe fn set_sql_state(sql_state: &str, output_ptr: *mut WChar) {
    if output_ptr.is_null() {
        return;
    }
    let sql_state = &format!("{}\0", sql_state);
    let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
    copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
}

///
/// set_output_wstring_helper writes [`message`] to the *WChar [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
unsafe fn set_output_wstring_helper(
    message: &[u16],
    output_ptr: *mut WChar,
    buffer_len: usize,
) -> (usize, SqlReturn) {
    // If the output_ptr is null or no buffer space has been allocated, we need
    // to return SUCCESS_WITH_INFO.
    if output_ptr.is_null() || buffer_len == 0 {
        return (0usize, SqlReturn::SUCCESS_WITH_INFO);
    }
    // Check if the entire message plus a null terminator can fit in the buffer;
    // we should truncate the message if it's too long.
    let num_chars = min(message.len() + 1, buffer_len);
    // TODO SQL-1084: This will currently not work when we need to truncate data that takes more than
    // two bytes, such as emojis because it's assuming every character is 2 bytes.
    // Actually, this is not clear now. The spec suggests it may be up to the user to correctly
    // reassemble parts.
    copy_nonoverlapping(message.as_ptr(), output_ptr, num_chars - 1);
    *output_ptr.add(num_chars - 1) = 0u16;
    // return the number of characters in the message string, excluding the
    // null terminator
    if num_chars < message.len() {
        (num_chars - 1, SqlReturn::SUCCESS_WITH_INFO)
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
    // Check if the entire message plus a null terminator can fit in the buffer;
    // we should truncate the message if it's too long.
    let num_chars = min(message.len() + 1, buffer_len);
    copy_nonoverlapping(message.as_ptr(), output_ptr, num_chars - 1);
    *output_ptr.add(num_chars - 1) = 0u8;
    // return the number of characters in the message string, excluding the
    // null terminator
    if num_chars < message.len() {
        (num_chars - 1, SqlReturn::SUCCESS_WITH_INFO)
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
    if output_ptr.is_null() {
        return (data.len(), SqlReturn::SUCCESS_WITH_INFO);
    }
    // Check if the entire message can fit in the buffer;
    // we should truncate the message if it's too long.
    let data_len = data.len();
    let num_bytes = min(data_len, buffer_len);
    // It is possible that no buffer space has been allocated.
    if num_bytes == 0 {
        return (0, SqlReturn::SUCCESS_WITH_INFO);
    }
    copy_nonoverlapping(data.as_ptr(), output_ptr as *mut _, num_bytes);
    // return the number of characters in the binary
    if num_bytes < data_len {
        (num_bytes, SqlReturn::SUCCESS_WITH_INFO)
    } else {
        (num_bytes, SqlReturn::SUCCESS)
    }
}

pub mod i16_len {
    use super::*;
    ///
    /// set_output_wstring writes [`message`] to the *WChar [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring(
        message: &str,
        output_ptr: *mut WChar,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        let message = message.encode_utf16().collect::<Vec<u16>>();
        let (len, ret) = set_output_wstring_helper(&message, output_ptr, buffer_len);
        *text_length_ptr = len as SmallInt;
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
        message: &str,
        output_ptr: *mut Char,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        // TODO SQL-1087: consider encoding utf-8 using the encoding crate. This allows for somewhat
        // sensible output for characters in unicode - ascii.:writes
        let (len, ret) = set_output_string_helper(message.as_bytes(), output_ptr, buffer_len);
        *text_length_ptr = len as SmallInt;
        ret
    }
}

pub mod i32_len {
    use super::*;
    ///
    /// set_output_wstring writes [`message`] to the *WChar [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring(
        message: &str,
        output_ptr: *mut WChar,
        buffer_len: usize,
        text_length_ptr: *mut Integer,
    ) -> SqlReturn {
        let (len, ret) = set_output_wstring_helper(
            &message.encode_utf16().collect::<Vec<_>>(),
            output_ptr,
            buffer_len,
        );
        *text_length_ptr = len as Integer;
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
    #[allow(dead_code)]
    pub unsafe fn set_output_string(
        message: &str,
        output_ptr: *mut Char,
        buffer_len: usize,
        text_length_ptr: *mut Integer,
    ) -> SqlReturn {
        let (len, ret) = set_output_string_helper(message.as_bytes(), output_ptr, buffer_len);
        *text_length_ptr = len as Integer;
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
        if !data_len_ptr.is_null() {
            // If the output_ptr is NULL, we should still return the length of the message.
            *data_len_ptr = size_of::<T>() as i32;
        }
        if output_ptr.is_null() {
            return SqlReturn::SUCCESS_WITH_INFO;
        }
        copy_nonoverlapping(data as *const _, output_ptr as *mut _, 1);
        SqlReturn::SUCCESS
    }
}

pub mod isize_len {
    use super::*;
    ///
    /// set_output_wstring writes [`message`] to the *WChar [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_wstring(
        message: Vec<u16>,
        col_num: USmallInt,
        index: usize,
        var_data_cache: &mut HashMap<USmallInt, CachedData>,
        output_ptr: *mut WChar,
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
            *text_length_ptr = 0;
            return SqlReturn::NO_DATA;
        }
        let (len, ret) =
            set_output_wstring_helper(message.get(index..).unwrap(), output_ptr, buffer_len);
        // the returned length should always be the total length of the data.
        *text_length_ptr = (message.len() - index) as Len;
        var_data_cache.insert(col_num, CachedData::WChar(index + len, message));
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
        message: Vec<u8>,
        col_num: USmallInt,
        index: usize,
        var_data_cache: &mut HashMap<USmallInt, CachedData>,
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
            *text_length_ptr = 0;
            return SqlReturn::NO_DATA;
        }
        let (len, ret) =
            set_output_string_helper(message.get(index..).unwrap(), output_ptr, buffer_len);
        // the returned length should always be the total length of the data.
        *text_length_ptr = (message.len() - index) as Len;
        // The lenth parameter does not matter because character data uses 8bit words and
        // we can obtain it from message.chars().count() above.
        var_data_cache.insert(col_num, CachedData::Char(len + index, message));
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
        data: Vec<u8>,
        col_num: USmallInt,
        index: usize,
        var_data_cache: &mut HashMap<USmallInt, CachedData>,
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
            *text_length_ptr = 0;
            return SqlReturn::NO_DATA;
        }
        let (len, ret) =
            set_output_binary_helper(data.get(index..).unwrap(), output_ptr, buffer_len);
        *text_length_ptr = (data.len() - index) as Len;
        var_data_cache.insert(col_num, CachedData::Bin(len + index, data));
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
        if !data_len_ptr.is_null() {
            // If the output_ptr is NULL, we should still return the length of the message.
            *data_len_ptr = size_of::<T>() as isize;
        }
        copy_nonoverlapping(data as *const _, output_ptr as *mut _, 1);
        SqlReturn::SUCCESS
    }
}

///
/// get_diag_rec copies the given ODBC error's diagnostic information
/// into the provided pointers.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn get_diag_rec(
    error: &ODBCError,
    state: *mut WChar,
    message_text: *mut WChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
    native_error_ptr: *mut Integer,
) -> SqlReturn {
    if !native_error_ptr.is_null() {
        *native_error_ptr = error.get_native_err_code();
    }
    set_sql_state(error.get_sql_state(), state);
    let message = format!("{}", error);
    i16_len::set_output_wstring(
        &message,
        message_text,
        buffer_length as usize,
        text_length_ptr,
    )
}

///
/// unsupported_function is a helper function for correctly setting the state for
/// unsupported functions.
///
pub fn unsupported_function(handle: &mut MongoHandle, name: &'static str) -> SqlReturn {
    handle.clear_diagnostics();
    handle.add_diag_info(ODBCError::Unimplemented(name));
    SqlReturn::ERROR
}

///
/// set_str_length writes the given length to [`string_length_ptr`].
///
/// # Safety
/// This writes to a raw C-pointers
///
pub unsafe fn set_str_length(string_length_ptr: *mut Integer, length: Integer) {
    if !string_length_ptr.is_null() {
        *string_length_ptr = length
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use bson::bson;
    #[test]
    fn date_format() {
        assert_eq!(
            Utc.timestamp(1003483404, 123000000),
            bson!("2001-10-19T09:23:24.123Z").to_datetime().unwrap()
        );
    }

    mod conversion {
        use crate::api::data::IntoCData;
        use bson::Bson;
        use constants::{FRACTIONAL_TRUNCATION, INTEGRAL_TRUNCATION};
        #[test]
        fn positive_values_without_truncation_to_u64_succeed() {
            let bson_double_to_u64 = Bson::Double(42.).to_u64();
            let bson_int64_to_u64 = Bson::Int64(42).to_u64();
            let bson_int32_to_u64 = Bson::Int32(42).to_u64();

            assert_eq!(bson_double_to_u64.unwrap().0, 42);
            assert_eq!(bson_int64_to_u64.unwrap().0, 42);
            assert_eq!(bson_int32_to_u64.unwrap().0, 42);
        }

        #[test]
        fn positive_values_without_truncation_to_i64_succeed() {
            let bson_double_to_i64 = Bson::Double(42.).to_i64();
            let bson_int64_to_i64 = Bson::Int64(42).to_i64();
            let bson_int32_to_i64 = Bson::Int32(42).to_i64();

            assert_eq!(bson_double_to_i64.unwrap().0, 42);
            assert_eq!(bson_int64_to_i64.unwrap().0, 42);
            assert_eq!(bson_int32_to_i64.unwrap().0, 42);
        }

        #[test]
        fn positive_values_without_truncation_to_u32_succeed() {
            let bson_double_to_u32 = Bson::Double(42.).to_u32();
            let bson_int64_to_u32 = Bson::Int64(42).to_u32();
            let bson_int32_to_u32 = Bson::Int32(42).to_u32();

            assert_eq!(bson_double_to_u32.unwrap().0, 42);
            assert_eq!(bson_int64_to_u32.unwrap().0, 42);
            assert_eq!(bson_int32_to_u32.unwrap().0, 42);
        }

        #[test]
        fn positive_values_without_truncation_to_i32_succeed() {
            let bson_double_to_i32 = Bson::Double(42.).to_u32();
            let bson_int64_to_i32 = Bson::Int64(42).to_u32();
            let bson_int32_to_i32 = Bson::Int32(42).to_u32();

            assert_eq!(bson_double_to_i32.unwrap().0, 42);
            assert_eq!(bson_int64_to_i32.unwrap().0, 42);
            assert_eq!(bson_int32_to_i32.unwrap().0, 42);
        }

        #[test]
        fn positive_values_with_fractional_truncation_to_u64_succeed_with_info() {
            let bson_double_to_u64 = Bson::Double(42.05).to_u64();

            assert_eq!(bson_double_to_u64.as_ref().unwrap().0, 42);
            assert_eq!(
                bson_double_to_u64.unwrap().1.unwrap().get_sql_state(),
                FRACTIONAL_TRUNCATION
            );
        }

        #[test]
        fn positive_values_with_fractional_truncation_to_i64_succeed_with_info() {
            let bson_double_to_i64 = Bson::Double(42.05).to_i64();

            assert_eq!(bson_double_to_i64.as_ref().unwrap().0, 42);
            assert_eq!(
                bson_double_to_i64.unwrap().1.unwrap().get_sql_state(),
                FRACTIONAL_TRUNCATION
            );
        }

        #[test]
        fn positive_values_with_fractional_truncation_to_u32_succeed_with_info() {
            let bson_double_to_u32 = Bson::Double(42.05).to_i64();

            assert_eq!(bson_double_to_u32.as_ref().unwrap().0, 42);
            assert_eq!(
                bson_double_to_u32.unwrap().1.unwrap().get_sql_state(),
                FRACTIONAL_TRUNCATION
            );
        }

        #[test]
        fn positive_values_with_fractional_truncation_to_i32_succeed_with_info() {
            let bson_double_to_i32 = Bson::Double(42.05).to_i64();

            assert_eq!(bson_double_to_i32.as_ref().unwrap().0, 42);
            assert_eq!(
                bson_double_to_i32.unwrap().1.unwrap().get_sql_state(),
                FRACTIONAL_TRUNCATION
            );
        }

        #[test]
        fn values_with_integral_truncation_to_u32_fail() {
            let bson_double_to_u32 = Bson::Double(f64::MAX - 0.1).to_u32();
            let bson_int64_to_u32 = Bson::Int64(i64::MAX).to_u32();

            assert_eq!(
                bson_double_to_u32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION,
                "expected integral truncation error"
            );

            assert_eq!(
                bson_int64_to_u32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION,
                "expected integral truncation error"
            );
        }

        #[test]
        fn values_with_integral_truncation_to_i32_fail() {
            let bson_double_to_i32 = Bson::Double(f64::MAX - 0.1).to_i32();
            let bson_int64_to_i32 = Bson::Int64(i64::MAX).to_i32();

            assert_eq!(
                bson_double_to_i32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION,
                "expected integral truncation error"
            );

            assert_eq!(
                bson_int64_to_i32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION,
                "expected integral truncation error"
            );
        }

        #[test]
        fn negative_values_to_u32_fail() {
            let bson_double_to_u32 = Bson::Double(-42.).to_u32();
            let bson_int64_to_u32 = Bson::Int64(-42).to_u32();
            let bson_int32_to_u32 = Bson::Int32(-42).to_u32();

            assert!(bson_double_to_u32.is_err());
            assert!(bson_int64_to_u32.is_err());
            assert!(bson_int32_to_u32.is_err());

            assert_eq!(
                bson_double_to_u32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
            assert_eq!(
                bson_int64_to_u32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
            assert_eq!(
                bson_int32_to_u32.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
        }

        #[test]
        fn negative_values_to_u64_fail() {
            let bson_double_to_u64 = Bson::Double(-42.).to_u32();
            let bson_int64_to_u64 = Bson::Int64(-42).to_u32();
            let bson_int32_to_u64 = Bson::Int32(-42).to_u32();

            assert!(bson_double_to_u64.is_err());
            assert!(bson_int64_to_u64.is_err());
            assert!(bson_int32_to_u64.is_err());

            assert_eq!(
                bson_double_to_u64.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
            assert_eq!(
                bson_int64_to_u64.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
            assert_eq!(
                bson_int32_to_u64.unwrap_err().get_sql_state(),
                INTEGRAL_TRUNCATION
            );
        }
    }

    // This just checks that f64 parsing can handle our output Decimal128 strings.
    mod decimal128_to_f64 {
        use mongo_odbc_core::util::Decimal128Plus;
        use std::str::FromStr;

        #[test]
        fn nan() {
            assert!(f64::from_str(
                &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124].to_formatted_string(),
            )
            .unwrap()
            .is_nan());
        }

        #[test]
        fn inf() {
            assert_eq!(
                f64::INFINITY,
                f64::from_str(
                    &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_inf() {
            assert_eq!(
                f64::NEG_INFINITY,
                f64::from_str(
                    &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn zero() {
            assert_eq!(
                0.0,
                f64::from_str(
                    &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 48].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_zero() {
            assert_eq!(
                0.0,
                f64::from_str(
                    &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 176].to_formatted_string()
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
                    &[1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 48].to_formatted_string()
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
                    &[1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 176].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn big() {
            assert_eq!(
                412345123451234512345.0,
                f64::from_str(
                    &[217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 48]
                        .to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_big() {
            assert_eq!(
                -412345123451234512345.0,
                f64::from_str(
                    &[217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 176]
                        .to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn really_big() {
            assert_eq!(
                1.8E+305,
                f64::from_str(
                    &[18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 50].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_really_big() {
            assert_eq!(
                -1.8E+305,
                f64::from_str(
                    &[18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 178].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn really_small() {
            assert_eq!(
                1.8E-305,
                f64::from_str(
                    &[18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 45].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_really_small() {
            assert_eq!(
                -1.8E-305,
                f64::from_str(
                    &[18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 173].to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn pi() {
            assert_eq!(
                std::f64::consts::PI,
                f64::from_str(
                    &[96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 48]
                        .to_formatted_string()
                )
                .unwrap(),
            );
        }

        #[test]
        fn neg_pi() {
            assert_eq!(
                -std::f64::consts::PI,
                f64::from_str(
                    &[96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 176]
                        .to_formatted_string()
                )
                .unwrap(),
            );
        }
    }
}

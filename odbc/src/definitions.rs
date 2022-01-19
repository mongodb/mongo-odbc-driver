use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum SqlBool {
    SqlFalse = 0,
    SqlTrue,
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum OdbcVersion {
    Odbc3 = 3,
    Odbc3_80 = 380,
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum ConnectionPooling {
    Off = 0,
    OnePerDriver,
    OnePerHEnv,
    DriverAware,
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum CpMatch {
    Strict = 0,
    Relaxed,
}

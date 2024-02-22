mod unit {
    // todo:
    //  - Invalid Option -> SqlReturn::ERROR
    //  - SQL_RESET_PARAMS -> SqlReturn::SUCCESS
    //  - SQL_UNBIND
    //      -> set up with Some(bound_cols)
    //      -> SqlReturn::SUCCESS
    //      -> Check that bound_cols is None afterwards
    //  - SQL_CLOSE
    //      -> set up with Some(current) and Some(resultset_cursor)
    //      -> SqlReturn::SUCCESS
    //      -> Check that current and resultset_cursor are None (and other fields are unchanged)
}

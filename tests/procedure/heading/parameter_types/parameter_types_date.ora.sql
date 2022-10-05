-- test: parameter types DATE
CREATE OR REPLACE PROCEDURE parameter_types_date(
    p_date DATE
    , p_timestamp TIMESTAMP
    , p_timestamp_with_time_zone TIMESTAMP WITH TIME ZONE
    , p_interval_year_to_month INTERVAL YEAR TO MONTH
    , p_interval_day_to_second INTERVAL DAY TO SECOND
    , p_timestamp_with_local_time_zeon TIMESTAMP WITH LOCAL TIME ZONE
    , p_time_unconstrained TIME_UNCONSTRAINED
    , p_time_tz_unconstrained TIME_TZ_UNCONSTRAINED
    , p_timestamp_unconstrained TIMESTAMP_UNCONSTRAINED
    , p_timestamp_tz_unconstrained TIMESTAMP_TZ_UNCONSTRAINED
    , p_yminterval_unconstrained YMINTERVAL_UNCONSTRAINED
    , p_dsinterval_unconstrained DSINTERVAL_UNCONSTRAINED
    , p_timestamp_ltz_unconstrained TIMESTAMP_LTZ_UNCONSTRAINED
)
IS
BEGIN
    NULL;
END parameter_types_date;

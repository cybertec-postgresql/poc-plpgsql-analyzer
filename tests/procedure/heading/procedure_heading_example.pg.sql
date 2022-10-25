-- test: example for procedure heading
CREATE OR REPLACE PROCEDURE procedure_heading_example (
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    p_1 INOUT text
    , p_2 bigint
    , p_3 boolean DEFAULT FALSE
    -- IN parameter after one with default value must also have defaults
    , p_4 INOUT timestamp DEFAULT NULL
    , p_5 foo.bar%TYPE DEFAULT NULL
)
AS $$
BEGIN
    NULL;
END;
$$ LANGUAGE plpgsql;

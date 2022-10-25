-- test: example for function heading
CREATE OR REPLACE FUNCTION function_heading_example (
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    p_1 OUT VARCHAR2
    , p_2 NUMBER
    , p_3 IN BOOLEAN := FALSE
    -- IN parameter after one with default value must also have defaults
    , p_4 IN OUT NOCOPY DATE
    , p_5 foo.bar%TYPE
)
RETURN NUMBER
IS
BEGIN
    RETURN 1;
END function_heading_example;

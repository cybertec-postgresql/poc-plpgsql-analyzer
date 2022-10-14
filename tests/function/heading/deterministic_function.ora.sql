-- test: DETERMINISTIC function
CREATE OR REPLACE FUNCTION deterministic_function
RETURN NUMBER DETERMINISTIC
IS
BEGIN
    RETURN 1;
END deterministic_function;

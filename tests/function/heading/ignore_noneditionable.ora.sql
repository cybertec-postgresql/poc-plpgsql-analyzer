-- test: ignore NONEDITIONABLE keyword, there is no equivalent in PostgreSQL
CREATE OR REPLACE NONEDITIONABLE FUNCTION ignore_noneditionable
RETURN number IS
BEGIN
 RETURN 1;
END;

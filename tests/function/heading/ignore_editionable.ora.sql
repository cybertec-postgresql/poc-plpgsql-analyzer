-- test: ignore EDITIONABLE keyword, there is no equivalent in PostgreSQL
CREATE OR REPLACE EDITIONABLE FUNCTION ignore_editionable
RETURN number IS
BEGIN
 RETURN 1;
END;

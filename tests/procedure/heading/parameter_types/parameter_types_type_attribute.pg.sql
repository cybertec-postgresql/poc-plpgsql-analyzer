-- test: define parameter type with column %TYPE attribute
-- pre-condition: CREATE TABLE foo (id INTEGER, bar VARCHAR2(20));
CREATE OR REPLACE PROCEDURE parameter_types_type_attribute(foo_bar foo.bar%TYPE)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;

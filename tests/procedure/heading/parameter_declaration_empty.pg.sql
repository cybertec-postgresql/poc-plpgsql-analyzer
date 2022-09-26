-- test: procedure with no parameter declaration
-- Note: providing parenthesis with no parameters leads to compilation error
CREATE OR REPLACE PROCEDURE parameter_declaration_empty()
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;

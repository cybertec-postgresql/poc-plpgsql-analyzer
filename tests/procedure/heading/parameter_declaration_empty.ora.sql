-- test: procedure with no parameter declaration
-- Note: providing parenthesis with no parameters leads to compilation error
CREATE OR REPLACE PROCEDURE parameter_declaration_empty
IS
BEGIN
    NULL;
END parameter_declaration_empty;

-- test: function with no parameter declaration
-- Note: providing parenthesis with no parameters leads to compilation error
CREATE OR REPLACE FUNCTION parameter_declaration_empty
RETURN NUMBER
IS
BEGIN
    RETURN 1;
END parameter_declaration_empty;

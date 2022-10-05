-- test: parameter types blob
CREATE OR REPLACE PROCEDURE parameter_types_blob(
    p_blob BLOB
    , p_binary_large_object BINARY LARGE OBJECT
)
IS
BEGIN
    NULL;
END parameter_types_blob;

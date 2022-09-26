-- test: multiple parameters with default values
CREATE OR REPLACE PROCEDURE multiple_parameters_default (
    p1 text
    , p2 text DEFAULT 'not empty'
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;

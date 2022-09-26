-- test: multiple parameters
CREATE OR REPLACE PROCEDURE multiple_parameters(
    p1 text
    , p2 text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;

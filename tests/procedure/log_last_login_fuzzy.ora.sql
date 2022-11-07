CREATE OR REPLACE PROCEDURE log_last_login_fuzzy (
    name persons.name%TYPE,
    number_of_logins persons.number_of_logins%TYPE,
    last_login persons.last_login%TYPE
)
IS
    formatted_output VARCHAR2(100);
BEGIN
    SELECT 'name: ' || trim(name) || ', number of logins: ' || trunc(number_of_logins) || ', last login: ' || trunc(last_login, 'MM') INTO formatted_output FROM DUAL;
    DBMS_OUTPUT.PUT_LINE(formatted_output);
END log_last_login_fuzzy;

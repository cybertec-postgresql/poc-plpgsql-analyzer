CREATE PACKAGE BODY northwind.util AS
    PROCEDURE print(str varchar2) IS
    BEGIN
        DBMS_OUTPUT.PUT_LINE('Output: ' || str);
    END;
END util;

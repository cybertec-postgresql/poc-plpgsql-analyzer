CREATE VIEW xmltype_view OF XMLTYPE
    XMLSCHEMA "http://www.plpgsql-analyzer.com/store.xsd"
    ELEMENT "Store"
    WITH OBJECT ID
    (extract(OBJECT_VALUE, '/Store/Location/text()'))
AS SELECT XMLELEMENT("Store",
     XMLFOREST(StoreID as "Building",
               area as "Area",
               parking as "Parking"))
   FROM stores;

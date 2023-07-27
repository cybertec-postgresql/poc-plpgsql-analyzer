CREATE OR REPLACE VIEW object_type_view OF employee_type
  WITH OBJECT ID (store_id)
  AS SELECT custom_type(store_id, employee_name, salary),
     FROM employees;

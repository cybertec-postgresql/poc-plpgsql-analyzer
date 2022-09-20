CREATE FUNCTION update_job_history()
  RETURNS TRIGGER
  LANGUAGE PLPGSQL
AS $$
BEGIN
  add_job_history(:old.employee_id, :old.hire_date, sysdate,
                  :old.job_id, :old.department_id);
END;
$$

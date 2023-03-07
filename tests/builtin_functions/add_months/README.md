# add_months()

Adds or Substitute Months from a given Date/Timestamp.

## Ora:
add_months(sysdate, 1);
add_months('2023-01-01, -4);

## PG:
SELECT current_timestamp + interval '1 months';
SELECT DATE '2023-01-01' - interval '4 months';

add_months()

ADD_MONTHS('07-03-2023', 1)
ADD_MONTHS('07-03-2023', -4)

Adds or substitute n months to a specific date/timestamp. The PG workaround could be 

"+ interval '1 month';"
"- interval '4 months';"

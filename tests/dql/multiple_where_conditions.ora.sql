SELECT *
FROM a, b, c
WHERE 100 < a
  AND (b <= 50 OR c LIKE '%foo%')
;

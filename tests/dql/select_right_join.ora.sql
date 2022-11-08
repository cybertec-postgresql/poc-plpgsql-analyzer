SELECT *
FROM persons, places
WHERE
  -- RIGHT (OUTER) JOIN
  persons.id(+) = places.person_id;
  -- Can be switched, still the same
  -- places.person_id = persons.id(+);

SELECT *
FROM persons, places
WHERE
  -- LEFT (OUTER) JOIN
  places.person_id(+) = persons.id;
  -- Can be switched, still the same
  -- persons.id = places.person_id(+);
  --
  -- Valid syntax: whitespaces may be used between the (+) an the column
  -- places.person_id (+) = persons.id;
  -- places.person_id (+)= persons.id;

# Translate the blind reading to the existing independent observed-response grader.
# The added rationale describes the already frozen semantic classification.
map(.why = (
  if .status=="supported" then "The requested person's sentence names the full object: " + .answer + "."
  elif .status=="unmentioned" then "No sentence names the requested person " + .person + "."
  elif .status=="conflict" then .person + " has competing entries naming different objects."
  elif .status=="unspecified" then "The question gives no person to choose between the entries."
  else error("unrecognized blind reading") end)
  | if .status=="unmentioned" then .status="absent" else . end)

graph [
node
[
  id 0
  label "::Get the current time"
graphics [
  type "oval"
]
]
node
[
  id 1
  label "::Is the current time is divisible by 2?"
graphics [
  type "oval"
]
]
node
[
  id 2
  label "::Program finished"
graphics [
  type "oval"
]
]
node
[
  id 3
  label "::Start the program"
graphics [
  type "oval"
]
]
node
[
  id 4
  label "::divisible::enter"
graphics [
  type "oval"
]
]
node
[
  id 5
  label "::divisible::loop and print"
graphics [
  type "oval"
]
]
node
[
  id 6
  label "TERMINAL"
graphics [
  type "rhombus"
]
]
node
[
  id 7
  label "INITIAL"
graphics [
  type "rectangle"
]
]
edge
[
  source 0
  target 1
  label "P: 1.00"
,]
edge
[
  source 1
  target 4
  label "P: 1.00"
,]
edge
[
  source 2
  target 6
  label "P: 1.00"
,]
edge
[
  source 3
  target 0
  label "P: 1.00"
,]
edge
[
  source 4
  target 5
  label "P: 1.00"
,]
edge
[
  source 5
  target 2
  label "P: 0.10"
,]
edge
[
  source 5
  target 5
  label "P: 0.90"
,]
edge
[
  source 7
  target 3
  label "P: 1.00"
,]
]

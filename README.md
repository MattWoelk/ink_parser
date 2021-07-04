supported [features](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md):

## v0.1

- [x] content
- [ ] comments
- [ ] tags
  - [x] multiple same-line tags
  - [ ] "Tags for a line can be written above it, or on the end of the line"
  - [ ] tags above the first line of a knot are also tags on that knot
  - [ ] global tags (at the very top of the main ink file)
- [x] choices (+)
- [x] sticky choices as special (+)
- [ ] choices (*)
- [x] knots
- [x] knot titles with trailing ='s
- [x] diverts
- [ ] choices, content, and diverts on the same line
- [ ] stitches
- [ ] local diverts
- [ ] fallback choice (choice without choice text)

## v0.2
- [ ] conditional choices `{}`
- [ ] logical operators AND `&&`, OR `||`, and NOT `not`
- [ ] not as exclamation point: `!`
- [ ] integer comparison checks `{seen_clue > 3}`
- [ ] conditional text `{variable: text if true|text if false}`
- [ ] global variables `VAR`
- [ ] numerical maths and logic `~ x = (x*x) - (y*y)`

## v0.3
- [ ] glue (though maybe this is more part of the story runner?)
- [ ] includes
- [ ] alternatives: sequences `|`
- [ ] alternatives: cycles `&`
- [ ] alternatives: once-only `!`
- [ ] alternatives: shuffles `~`
- [ ] alternatives: blank elements
- [ ] alternatives: nested
- [ ] alternatives: divert statements
- [ ] alternatives: inside choice text
- [ ] alternatives: escaping `{` with backslash
- [ ] CHOICE_COUNT()
- [ ] TURNS()
- [ ] TURNS_SINCE()
- [ ] SEED_RANDOM()
- [ ] storing diverts as variables
- [ ] printing variables
- [ ] evaluating strings
- [ ] RANDOM()
- [ ] INT() FLOOR() FLOAT()
- [ ] string comparison `==`, `!=`, `?`
- [ ] conditional blocks `if`, `else`
- [ ] switch blocks
- [ ] temporary variables
- [ ] knot and stitch parameters

## v0.4

- [ ] functions
- [ ] global constants
- [ ] tunnels
- [ ] threads
- [ ] lists (TODO: split this into sub-sections)
- [ ] Weave: gathers
- [ ] Weave: nested flow
- [ ] Weave: nested gather points
- [ ] Weave: labelled gather points and options `- (label)`
- [ ] Weave: scope

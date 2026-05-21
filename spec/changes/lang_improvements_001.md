# Calc Language Improvements 001
## Purpose
- To allow a reference to a variable to recognize a section that is at the same level
```calc
level0:
  level1:
    var = 0
  var_plus_one = level1.var + 1
```
- To implement error messages for lexing, parsing and evaluation errors. Evaluate how suitable error messages for Zed could be implemented
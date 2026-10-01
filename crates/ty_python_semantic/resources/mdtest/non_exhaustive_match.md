# Non-exhaustive match statements

```toml
[environment]
python-version = "3.11"

[rules]
non-exhaustive-match = "error"
```

## Diagnostic

```py
from typing import Literal

def describe(value: Literal["red", "green"]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case "red":
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: value `"green"` is not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal["red", "green"]`
help: Add a `case` branch for the remaining values
  |
5 |         case "red":
  -             pass
6 +             pass
7 +         case "green":
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Enum diagnostics

The diagnostic lists at most three missing members and points to their definitions.

```py
from enum import Enum

class Direction(Enum):
    NORTH = 1
    SOUTH = 2
    EAST = 3
    WEST = 4
    UP = 5

def describe(value: Direction) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Direction.NORTH:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `Direction.SOUTH`, `Direction.EAST`, `Direction.WEST` and 1 more are not covered
  --> src/mdtest_snippet.py:11:11
   |
11 |     match value:  # snapshot: non-exhaustive-match
   |           ^^^^^ Subject has type `Direction`
   |
  ::: src/mdtest_snippet.py:5:5
   |
 5 |     SOUTH = 2
   |     ----- enum variant `SOUTH` is not covered
 6 |     EAST = 3
   |     ---- enum variant `EAST` is not covered
 7 |     WEST = 4
   |     ---- enum variant `WEST` is not covered
info: Use `--verbose` to see all 4 uncovered values
help: Add a `case` branch for the remaining values
   |
12 |         case Direction.NORTH:
   -             pass
13 +             pass
14 +         case Direction.SOUTH | Direction.EAST | Direction.WEST | Direction.UP:
15 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Verbose enum diagnostics

With `--verbose`, the diagnostic lists all missing members.

```toml
verbose = true

[environment]
python-version = "3.11"

[rules]
non-exhaustive-match = "error"
```

```py
from enum import Enum

class Direction(Enum):
    NORTH = 1
    SOUTH = 2
    EAST = 3
    WEST = 4
    UP = 5

def describe(value: Direction) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Direction.NORTH:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `Direction.SOUTH`, `Direction.EAST`, `Direction.WEST` and `Direction.UP` are not covered
  --> src/mdtest_snippet.py:11:11
   |
 5 |     SOUTH = 2
   |     ----- enum variant `SOUTH` is not covered
 6 |     EAST = 3
   |     ---- enum variant `EAST` is not covered
 7 |     WEST = 4
   |     ---- enum variant `WEST` is not covered
 8 |     UP = 5
   |     -- enum variant `UP` is not covered
 9 |
10 | def describe(value: Direction) -> None:
11 |     match value:  # snapshot: non-exhaustive-match
   |           ^^^^^ Subject has type `Direction`
help: Add a `case` branch for the remaining values
info: rule `non-exhaustive-match` was selected in the configuration file
   |
12 |         case Direction.NORTH:
   -             pass
13 +             pass
14 +         case Direction.SOUTH | Direction.EAST | Direction.WEST | Direction.UP:
15 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Enum and literal unions

```py
from enum import Enum
from typing import Literal

class Color(Enum):
    RED = 1
    BLUE = 2

def describe(value: Color | Literal["stop"]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `Color.BLUE` and `"stop"` are not covered
 --> src/mdtest_snippet.py:9:11
  |
6 |     BLUE = 2
  |     ---- enum variant `Color.BLUE` is not covered
7 |
8 | def describe(value: Color | Literal["stop"]) -> None:
9 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color | Literal["stop"]`
help: Add a `case` branch for the remaining values
   |
10 |         case Color.RED:
   -             pass
11 +             pass
12 +         case Color.BLUE | "stop":
13 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Literal unions

```py
from typing import Literal

def describe(value: Literal["red", "green", "blue"]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case "red":
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `"green"` and `"blue"` are not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal["red", "green", "blue"]`
help: Add a `case` branch for the remaining values
  |
5 |         case "red":
  -             pass
6 +             pass
7 +         case "green" | "blue":
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## More than three missing literal values

```py
from typing import Literal

def describe(value: Literal["red", "green", "blue", "white", "black"]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case "red":
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `"green"`, `"blue"`, `"white"` and 1 more are not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal["red", "green", "blue", "white", "black"]`
info: Use `--verbose` to see all 4 uncovered values
help: Add a `case` branch for the remaining values
  |
5 |         case "red":
  -             pass
6 +             pass
7 +         case "green" | "blue" | "white" | "black":
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Five missing literal values

```py
from typing import Literal

def describe(value: Literal[0, 1, 2, 3, 4, 5]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case 0:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `1`, `2`, `3` and 2 more are not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal[0, 1, 2, 3, 4, 5]`
info: Use `--verbose` to see all 5 uncovered values
help: Add a `case` branch for the remaining values
  |
5 |         case 0:
  -             pass
6 +             pass
7 +         case 1 | 2 | 3 | 4 | 5:
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## More than five missing literal values

```py
from typing import Literal

def describe(value: Literal[0, 1, 2, 3, 4, 5, 6]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case 0:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `1`, `2`, `3` and 3 more are not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal[0, 1, 2, 3, 4, 5, 6]`
info: Use `--verbose` to see all 6 uncovered values
help: Add a `case` branch for the remaining values
  |
5 |         case 0:
  -             pass
6 +             pass
7 +         case _:
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Functional enum

```py
from enum import Enum

Color = Enum("Color", "RED GREEN BLUE")

def describe(value: Color) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `Color.GREEN` and `Color.BLUE` are not covered
 --> src/mdtest_snippet.py:6:11
  |
3 | Color = Enum("Color", "RED GREEN BLUE")
  |                       ---------------- Enum variants `GREEN` and `BLUE` are not covered
4 |
5 | def describe(value: Color) -> None:
6 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color`
help: Add a `case` branch for the remaining values
   |
7  |         case Color.RED:
   -             pass
8  +             pass
9  +         case Color.GREEN | Color.BLUE:
10 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Functional enum in a union

```py
from enum import Enum
from typing import Literal

Color = Enum("Color", "RED GREEN BLUE")

def describe(value: Color | Literal["stop"]) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `Color.GREEN`, `Color.BLUE` and `"stop"` are not covered
 --> src/mdtest_snippet.py:7:11
  |
4 | Color = Enum("Color", "RED GREEN BLUE")
  |                       ---------------- Enum variants `Color.GREEN` and `Color.BLUE` are not covered
5 |
6 | def describe(value: Color | Literal["stop"]) -> None:
7 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color | Literal["stop"]`
help: Add a `case` branch for the remaining values
   |
8  |         case Color.RED:
   -             pass
9  +             pass
10 +         case Color.GREEN | Color.BLUE | "stop":
11 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Functional enum with a keyword argument

```py
from enum import Enum

Color = Enum("Color", names={"RED": 1, "BLUE": 2})

def describe(value: Color) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: value `Color.BLUE` is not covered
 --> src/mdtest_snippet.py:6:11
  |
3 | Color = Enum("Color", names={"RED": 1, "BLUE": 2})
  |                             --------------------- Enum variant `BLUE` is not covered
4 |
5 | def describe(value: Color) -> None:
6 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color`
help: Add a `case` branch for the remaining values
   |
7  |         case Color.RED:
   -             pass
8  +             pass
9  +         case Color.BLUE:
10 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## A missing `None` case

```py
from typing import Literal

def describe(value: Literal["red"] | None) -> None:
    match value:  # snapshot: non-exhaustive-match
        case "red":
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: `None` is not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal["red"] | None`
help: Add a `case` branch for the remaining values
  |
5 |         case "red":
  -             pass
6 +             pass
7 +         case None:
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Enum members defined in another file

The suggested pattern uses the name by which the enum was imported.

`colors.py`:

```py
from enum import Enum

class Color(Enum):
    RED = 1
    BLUE = 2
```

```py
from colors import Color as Hue

def describe(value: Hue) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Hue.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: value `Color.BLUE` is not covered
 --> src/mdtest_snippet.py:4:11
  |
4 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color`
  |
 ::: src/colors.py:5:5
  |
5 |     BLUE = 2
  |     ---- enum variant `BLUE` is not covered
help: Add a `case` branch for the remaining values
  |
5 |         case Hue.RED:
  -             pass
6 +             pass
7 +         case Hue.BLUE:
8 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Enums with the same name

`first.py`:

```py
from enum import Enum

class Color(Enum):
    RED = 1
    BLUE = 2
```

`second.py`:

```py
from enum import Enum

class Color(Enum):
    RED = 1
    BLUE = 2
```

```py
import first
import second

def describe(value: first.Color | second.Color) -> None:
    match value:  # snapshot: non-exhaustive-match
        case first.Color.RED | second.Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: values `first.Color.BLUE` and `second.Color.BLUE` are not covered
 --> src/mdtest_snippet.py:5:11
  |
5 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `first.Color | second.Color`
  |
 ::: src/first.py:5:5
  |
5 |     BLUE = 2
  |     ---- enum variant `first.Color.BLUE` is not covered
  |
 ::: src/second.py:5:5
  |
5 |     BLUE = 2
  |     ---- enum variant `second.Color.BLUE` is not covered
help: Add a `case` branch for the remaining values
  |
6 |         case first.Color.RED | second.Color.RED:
  -             pass
7 +             pass
8 +         case first.Color.BLUE | second.Color.BLUE:
9 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Flag instances

Flag members do not exhaust the possible flag values.

```py
from enum import Flag

class Permission(Flag):
    READ = 1
    WRITE = 2

def describe(value: Permission) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Permission.READ:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: objects of type `Permission & ~Literal[Permission.READ]` are not covered
 --> src/mdtest_snippet.py:8:11
  |
8 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Permission`
info: `enum.Flag` can have unnamed combinations of members
info: See https://docs.python.org/3/howto/enum.html#combining-members-of-flag
help: Add a `case` branch for the remaining values
   |
9  |         case Permission.READ:
   -             pass
10 +             pass
11 +         case _:
12 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Flag instances in a union

```py
from enum import IntFlag

class Permission(IntFlag):
    READ = 1
    WRITE = 2

def unhandled_flag_and_string(value: Permission | str | bytes) -> None:
    match value:  # snapshot: non-exhaustive-match
        case bytes():
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: objects of type `Permission | str` are not covered
 --> src/mdtest_snippet.py:8:11
  |
8 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Permission | str | bytes`
info: `enum.Flag` can have unnamed combinations of members
info: See https://docs.python.org/3/howto/enum.html#combining-members-of-flag
help: Add a `case` branch for the remaining values
   |
10 |             pass
11 +         case _:
12 +             raise NotImplementedError("TODO")
13 | def unhandled_string(value: Permission | str) -> None:
   |
note: This is a display-only fix and is likely to be incorrect
```

When the flag is already covered, the diagnostic does not include the explanation about flags.

```py
def unhandled_string(value: Permission | str) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Permission():
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: objects of type `str` are not covered
  --> src/mdtest_snippet.py:12:11
   |
12 |     match value:  # snapshot: non-exhaustive-match
   |           ^^^^^ Subject has type `Permission | str`
help: Add a `case` branch for the remaining values
   |
13 |         case Permission():
   -             pass
14 +             pass
15 +         case _:
16 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## A union containing an open type

```py
from enum import Enum

class Color(Enum):
    RED = 1
    BLUE = 2

def describe(value: Color | str) -> None:
    match value:  # snapshot: non-exhaustive-match
        case Color.RED:
            pass
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: objects of type `Literal[Color.BLUE] | str` are not covered
 --> src/mdtest_snippet.py:8:11
  |
8 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Color | str`
help: Add a `case` branch for the remaining values
   |
9  |         case Color.RED:
   -             pass
10 +             pass
11 +         case _:
12 +             raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Suggested wildcard case

```py
def describe(value: int | str) -> None:
    match value:  # snapshot: non-exhaustive-match
        case int():
            pass  # Kept with the existing case.
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: objects of type `str` are not covered
 --> src/mdtest_snippet.py:2:11
  |
2 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `int | str`
help: Add a `case` branch for the remaining values
  |
3 |         case int():
  -             pass  # Kept with the existing case.
4 +             pass  # Kept with the existing case.
5 +         case _:
6 +             raise NotImplementedError("TODO")
  |
note: This is a display-only fix and is likely to be incorrect
```

## Nested match with trailing comments

```py
from typing import Literal

def describe(value: Literal[1, 2], enabled: bool) -> None:
    if enabled:
        match value:  # snapshot: non-exhaustive-match
            case 1:
                pass
                # This comment belongs to the existing case.
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: value `2` is not covered
 --> src/mdtest_snippet.py:5:15
  |
5 |         match value:  # snapshot: non-exhaustive-match
  |               ^^^^^ Subject has type `Literal[1, 2]`
help: Add a `case` branch for the remaining values
   |
7  |                 pass
   -                 # This comment belongs to the existing case.
8  +                 # This comment belongs to the existing case.
9  +             case 2:
10 +                 raise NotImplementedError("TODO")
   |
note: This is a display-only fix and is likely to be incorrect
```

## Single-line case body

```py
from typing import Literal

def describe(value: Literal["red", "green"]) -> None:
    # fmt: off
    match value:  # snapshot: non-exhaustive-match
        case "red": pass  # Kept with the existing case.
    # fmt: on
```

```snapshot
error[non-exhaustive-match]: Match is not exhaustive: value `"green"` is not covered
 --> src/mdtest_snippet.py:5:11
  |
5 |     match value:  # snapshot: non-exhaustive-match
  |           ^^^^^ Subject has type `Literal["red", "green"]`
help: Add a `case` branch for the remaining values
  |
6 |         case "red": pass  # Kept with the existing case.
7 +         case "green":
8 +             raise NotImplementedError("TODO")
9 |     # fmt: on
  |
note: This is a display-only fix and is likely to be incorrect
```

## Literal patterns

```py
from typing import Literal

def incomplete(value: Literal["red", "green", "blue"]) -> None:
    match value:  # error: [non-exhaustive-match] "blue"
        case "red" | "green":
            pass

def complete(value: Literal["red", "green", "blue"]) -> None:
    match value:  # no diagnostic
        case "red" | "green":
            pass
        case "blue":
            pass

def booleans(value: bool) -> None:
    match value:  # error: [non-exhaustive-match] "Literal[False]"
        case True:
            pass

    match value:  # no diagnostic
        case True:
            pass
        case False:
            pass

def boolean_values_as_integers(value: bool) -> None:
    match value:  # no diagnostic
        case 0 | 1:
            pass

def wildcard(value: str) -> None:
    match value:  # no diagnostic
        case "red":
            pass
        case _:
            pass

def capture(value: str) -> None:
    match value:  # no diagnostic
        case captured:
            print(captured)
```

## Enum members

```py
from enum import Enum

class Color(Enum):
    RED = 1
    CRIMSON = 1
    GREEN = 2
    BLUE = 3

def incomplete(value: Color) -> None:
    match value:  # error: [non-exhaustive-match] "BLUE"
        case Color.RED | Color.GREEN:
            pass

def complete(value: Color) -> None:
    match value:  # no diagnostic
        case Color.RED | Color.GREEN:
            pass
        case Color.BLUE:
            pass

def alias(value: Color) -> None:
    match value:  # no diagnostic
        case Color.CRIMSON | Color.GREEN | Color.BLUE:
            pass
```

## Class patterns and unions

```py
def complete(value: int | str) -> None:
    match value:  # no diagnostic
        case int() | str():
            pass

def already_narrowed(value: int | str) -> None:
    if isinstance(value, int):
        match value:  # no diagnostic
            case int():
                pass

def open_type(value: int) -> None:
    match value:  # error: [non-exhaustive-match]
        case 1:
            pass
```

## Tuple patterns

```py
def incomplete(value: tuple[bool, bool]) -> None:
    match value:  # error: [non-exhaustive-match]
        case (True, _):
            pass
        case (False, True):
            pass

def complete(value: tuple[bool, bool]) -> None:
    match value:  # no diagnostic
        case (True, _):
            pass
        case (False, True):
            pass
        case (False, False):
            pass

def expression_subject(first: bool, second: bool) -> None:
    match (first, second):  # no diagnostic
        case (True, _):
            pass
        case (False, _):
            pass
```

## Guards

```py
from typing import Literal

def guarded(value: Literal[1, 2], flag: bool) -> None:
    match value:  # error: [non-exhaustive-match] "`1` is not covered"
        case 1 if flag:
            pass
        case 2:
            pass

    match value:  # no diagnostic
        case 1 if flag:
            pass
        case 1 | 2:
            pass

def true_guard(value: Literal[1, 2]) -> None:
    match value:  # no diagnostic
        case 1 if True:
            pass
        case 2:
            pass

def false_guard(value: Literal[1, 2]) -> None:
    match value:  # error: [non-exhaustive-match] "`1` is not covered"
        case 1 if False:
            pass
        case 2:
            pass

def guarded_wildcard(value: str, flag: bool) -> None:
    match value:  # error: [non-exhaustive-match]
        case _ if flag:
            pass

def true_guarded_wildcard(value: str) -> None:
    match value:  # no diagnostic
        case _ if True:
            pass

def subject_guard(value: Literal[1, 2]) -> None:
    match value:  # no diagnostic
        case 1 if value == 1:
            pass
        case 2:
            pass

def capture_guard(value: Literal[1, 2]) -> None:
    match value:  # no diagnostic
        case 1 as captured if captured == 1:
            pass
        case 2:
            pass

def final_capture_guard(value: Literal[1, 2]) -> None:
    match value:  # no diagnostic
        case 1:
            pass
        case 2 as captured if captured == 2:
            pass

def final_ambiguous_guard(value: Literal[1, 2], flag: bool) -> None:
    match value:  # error: [non-exhaustive-match] "`2` is not covered"
        case 1:
            pass
        case 2 if flag:
            pass

def changing_subject(value: int | str, again: bool) -> None:
    while again:
        match value:  # no diagnostic
            case str() as captured if captured is not None:
                value = 1
            case int():
                value = 0
```

## Uninhabited and dynamic subjects

```py
from typing import Any, Never

def uninhabited(value: Never) -> None:
    match value:  # no diagnostic
        case 1:
            pass

def dynamic(value: Any) -> None:
    match value:  # error: [non-exhaustive-match]
        case 1:
            pass

    match value:  # no diagnostic
        case _:
            pass

def unreachable(value: int) -> None:
    if False:
        match value:  # no diagnostic
            case 1:
                pass

def suppressed(value: int) -> None:
    match value:  # ty: ignore[non-exhaustive-match]
        case 1:
            pass
```

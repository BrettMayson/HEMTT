# Math

HEMTT includes a built-in mathematical expression evaluator used in config files. This allows you to use arithmetic operations, trigonometric functions, and angle conversions directly in config values, reducing the need for manual calculations or `__EVAL`.

The evaluator is automatically applied when parsing config files, allowing expressions like `value = 1 + 2;` to be evaluated to `value = 3;`.

## Supported Operators

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `2 + 3` → `5` |
| `-` | Subtraction | `5 - 2` → `3` |
| `*` | Multiplication | `2 * 3` → `6` |
| `/` | Division | `6 / 2` → `3` |
| `^` | Exponentiation | `2 ^ 3` → `8` |
| `%` | Modulo | `5 % 2` → `1` |

### Operator Precedence

Operations follow standard mathematical precedence:

1. Unary minus (negation): `-x`
2. Exponentiation: `^`
3. Multiplication, Division, Modulo: `*`, `/`, `%`
4. Addition, Subtraction: `+`, `-`

Parentheses can be used to override precedence:

```cpp
1 + 2 * 3        → 7
(1 + 2) * 3      → 9
```

## Functions

### Trigonometric Functions

All trigonometric functions work with radians.

| Function | Description | Example |
|----------|-------------|---------|
| `sin(x)` | Sine | `sin(0)` → `0` |
| `cos(x)` | Cosine | `cos(0)` → `1` |
| `tan(x)` or `tg(x)` | Tangent | `tan(0)` → `0` |
| `asin(x)` | Arcsine | `asin(1)` → `1.5708...` (π/2) |
| `acos(x)` | Arccosine | `acos(1)` → `0` |
| `atan(x)` or `atg(x)` | Arctangent | `atan(0)` → `0` |

### Angle Conversion Functions

| Function | Description | Example |
|----------|-------------|---------|
| `rad(x)` | Convert degrees to radians | `rad(180)` → `3.14159...` (π) |
| `deg(x)` | Convert radians to degrees | `deg(pi)` → `180` |

## Constants

| Constant | Value |
|----------|-------|
| `pi` | 3.14159265... |

## Examples

### Basic Calculations

```cpp
1 + 1                    → 2
10 - 3                   → 7
4 * 5                    → 20
20 / 4                   → 5
2 ^ 10                   → 1024
17 % 5                   → 2
```

### With Parentheses

```cpp
(1 + 2) * 3              → 9
1 + (2 * 3)              → 7
((2 + 3) * 4) - 1        → 19
```

### Negative Numbers

```cpp
-5                       → -5
1 + -2                   → -1
1 - -2                   → 3
2 * -(3 + 1)             → -8
```

### Trigonometric

```cpp
sin(0)                   → 0
cos(0)                   → 1
sin(pi / 2)              → 1
cos(pi)                  → -1
tan(0)                   → 0
```

### Angle Conversion

```cpp
rad(90)                  → 1.5708... (π/2)
rad(180)                 → 3.14159... (π)
deg(pi)                  → 180
deg(pi / 2)              → 90
```

### Complex Expressions

```cpp
2 * sin(pi / 6)          → 1 (sin(30°) = 0.5)
deg(atan(1))             → 45
cos(rad(60)) + sin(rad(30))  → 1
(pi * 2) ^ 2             → 39.478...
```

### With Constants

```cpp
pi                       → 3.14159...
2 * pi                   → 6.28318... (circumference factor)
pi * 5 ^ 2               → 78.539... (area of circle with radius 5)
```

## Arma Config Examples

Since this evaluator is used in Arma 3 config files, here are practical examples of how to use math expressions in your configurations:

### Basic Config Values

```cpp
class MyClass {
    // Simple arithmetic
    cost = 100 + 50;                     // Evaluates to 150
    multiplier = 2 * 3.5;                // Evaluates to 7
    
    // Damage calculations
    maxDamage = 100 / 2;                 // Evaluates to 50
    armorCoefficient = 1 / 0.8;          // Evaluates to 1.25
};
```

### Using Constants and Functions

```cpp
class MyWeapon {
    // Angles and rotations
    rotationAngle = rad(45);             // Convert 45° to radians
    deploymentAngle = deg(pi / 4);       // Convert π/4 radians to degrees
    
    // Trigonometric calculations
    offsetX = cos(rad(45)) * 2;
    offsetY = sin(rad(30)) * 5;
};
```

Invalid expressions will return an error:

```cpp
1 +                      → Error (incomplete expression)
(1 + 1                   → Error (mismatched parentheses)
1 & 1                    → Error (invalid operator)
```

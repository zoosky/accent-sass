---
name: Incorrect Sass Output
about: `accent-sass` and `dart-sass` differ in output or `accent-sass` reports and error for a valid style sheet
title: ''
labels: bug
assignees: connorskees

---

**Failing Sass**:
```
a {
  color: red;
}
```

<!-- Showing output from both tools is optional, but does help in debugging -->
**`accent-sass` Output**:
```
a {
  color: red;
}
```

**`dart-sass` Output**:
```
a {
  color: red;
}
```

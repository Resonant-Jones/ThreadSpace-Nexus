# 🧠 Mypy Fix Guide: `py3k.py`

## ✅ Goals
- Eliminate all mypy errors in this file
- Conform to `--strict` mode (e.g., `no_implicit_optional=True`)
- Ensure all function inputs and outputs are typed
- Annotate instance variables and collections

---

## 🔧 Fix Checklist

- [ ] Add `from __future__ import annotations`
- [ ] Import: `from typing import Optional, List, Dict, Any` (as needed)
- [ ] Add return types to all functions: `def foo(...) -> ReturnType:`
- [ ] Type instance variables: `self.var: Optional[str] = None`
- [ ] Fix `None` default args: `def bar(x: Optional[int] = None)`
- [ ] Use `List[...]` and `Dict[...]` for all collections
- [ ] Annotate model classes with `BaseModel` if using Pydantic
- [ ] Avoid bare `list`, `dict`, etc.

---

## 📁 File Summary

**File:** `.venv311/lib/python3.13/site-packages/numpy/compat/py3k.py`
**Lines:** `143`
**Suggested Priority:** ✴️ Medium

---

## 💡 Notes

Once all items are checked, verify with:

```bash
mypy .venv311/lib/python3.13/site-packages/numpy/compat/py3k.py --strict
```

And ensure no errors remain.

# 🧮 PyO3 Stage 3.1 Master Cheat Sheet — CPython C-API, Descriptors & FFI Memory Model

> **Primary Objective:** Within 30 seconds of inspecting any PyO3/CPython FFI interaction, trace the exact path from Rust macros (`#[pyclass]`, `#[pymethods]`) through CPython's `PyTypeObject` and `PyType_Ready()` into `Class.__dict__`, explain how descriptors cross the 16-byte Binary Wall via Rust trampolines, and execute zero-copy reference passing without double-allocation traps.

---

## 🧠 1. Core Mental Paradigm of the Domain

Think of a `#[pyclass]` instance on the Python heap as a **Diplomatic Envelope (`PyCell<T>` at `0x1000`)**:
* **The Outside of the Envelope (Bytes `0..16` — `PyObject` Header):** Written in Python's language (`ob_refcnt` and `ob_type`). Python can read the outside of the envelope, count how many people hold it, and look up its class dictionary (`Animal.__dict__`).
* **The Locked Vault Inside the Envelope (Bytes `16..N` — Rust Struct Payload):** Written in raw Rust binary (`age: i32 = 10`). **Python is 100% blind to the inside of the vault.**
* **The Translator Clerks (`Class.__dict__` Descriptors & Rust Trampolines):** Stored once at the Class level (`Animal.__dict__['age']`). When Python wants `dog.age`, it hands `dog`'s envelope (`0x1000`) to the clerk (`getset_descriptor`), who invokes a compiled **Rust Trampoline Function** that unlocks the vault at `+16 bytes`, reads the raw `i32` (`10`), mints a standard Python `int` (`PyLongObject`), and hands it back to Python.

Formal contrast: **Standard Python classes** store per-instance attributes in a dynamic `instance.__dict__` HashMap, whereas **PyO3 `#[pyclass]` structs** behave like Python's `__slots__`—storing raw binary fields inline inside `PyCell<T>` and exposing them through stateless C-level descriptors injected into `Class.__dict__` by `PyType_Ready()`.

---

## ⚙️ 2. Primitive Operations & Complexity Contract

Every FFI operation between Rust and Python has an exact C-API mechanism and memory cost.

| Operation | Formal Mechanism | What It Does Under the Hood | Complexity Target | Python Equivalent |
| :--- | :--- | :--- | :--- | :--- |
| **Register Class** | `m.add_class::<T>()` | Calls `PyType_Ready(&PyTypeObject)` to populate `Class.__dict__` from `tp_*` arrays, then inserts class into `Module.__dict__`. | $O(M + G)$ once at import ($M$ methods, $G$ getters) | Executing `class Animal: ...` inside a `.py` module |
| **Instantiate from Rust** | `Py::new(py, T { .. })` | Allocates `[PyObject Header \| PyCell<T>]` on the Python heap (`0x1000`) and returns a `Py<T>` handle. | $O(1)$ heap allocation | `dog = Animal.__new__(Animal)` |
| **Pass Handle to Python** | `func.call1(py, (&py_handle,))` | Increments `ob_refcnt` (`Py_INCREF`) on `0x1000` and passes the 8-byte pointer. **Zero struct copy.** | $O(1)$ time, $O(1)$ space | Passing `dog` reference to `func(dog)` |
| **Read Field (`dog.age`)** | `tp_getset` Trampoline | Python calls `Animal.__dict__['age'].__get__(0x1000)` $\rightarrow$ Rust reads offset $\rightarrow$ allocates `PyLongObject`. | $O(1)$ pointer offset + `PyLong` alloc | `__slots__` member descriptor lookup |
| **Call Method (`dog.speak()`)** | `tp_methods` Trampoline | Python calls `Animal.__dict__['speak'].__get__(0x1000)()` $\rightarrow$ Rust checks `PyCell` borrow flag $\rightarrow$ runs `&self`. | $O(1)$ dict lookup + borrow check | `Animal.__dict__['speak'].__get__(dog)()` |
| **Call Dunder (`str(dog)`)** | Direct `tp_str` VTable Slot | CPython jumps directly to `dog->ob_type->tp_str(0x1000)` (bypasses dict string hashing in C). | $O(1)$ direct C function pointer jump | `str(dog)` / `dog.__str__()` |

---

## 📈 3. Archetype Difficulty & Progression

Mastering Stage 3.1 requires progressing through five linked architectural layers, from surface macro syntax down to raw C-ABI memory layout.

```text
Title: Stage 3.1 Cognitive Progression Curve (From Macro Syntax to C-ABI Binary Layout)
========================================================================================
  Complexity ▲
             │                                                     [Phase 4: Boss]
             │                                                  Zero-Copy Handles &
             │                                       [Phase 3]   Borrow-Checked FFI
             │                                    The 16B Binary   (Py<T>, Bound,
             │                         [Phase 2]   Wall & Rust      PyCell RefCell)
             │                      PyType_Ready    Trampolines
             │           [Phase 1]  & Descriptor   (__pyo3_get_*)
             │         Macro-to-Dict   Bridge
             │           Mapping     (tp_* -> dict)
             └──────────────┴─────────────┴──────────────┴─────────────────► Phases
                         Gentle        Moderate        Steep          Mastery
========================================================================================
```

* **Phase 1 (Gentle — Macro-to-Dict Mapping):** Map `#[pymodule]`, `#[pyclass]`, and `#[pymethods]` to `Module.__dict__` and `Class.__dict__`.
* **Phase 2 (Moderate — The `PyType_Ready` Bridge):** Understand how compile-time C arrays (`tp_methods`, `tp_getset`) are converted into runtime descriptors inside `Class.__dict__`.
* **Phase 3 (Steep — The Binary Wall & Trampolines):** Trace how two instances (`dog` vs `cat`) keep separate binary state while sharing one descriptor in `Class.__dict__`, and how Rust trampolines cross the 16-byte header wall.
* **Phase 4 (Boss — Zero-Copy Lifecycle & Handles):** Distinguish primitive conversion copies (`ToPyObject` creating `PyLong`) from zero-copy handle passing (`Py<T>`, `Bound<'py, T>`).

---

## 🧭 4. 30-Second Decision Tree

Use this tree whenever you need to expose a Rust struct, method, or field to Python or pass data across the FFI boundary.

```text
Title: 30-Second PyO3 Stage 3.1 Architecture & FFI Decision Tree
========================================================================================
START: What FFI interaction or mental model are you resolving?
 │
 ├── 1. "How does Python find my Rust class, method, or field in __dict__?"
 │      └──► ARCHETYPE A: The PyTypeObject -> PyType_Ready() -> Class.__dict__ Bridge
 │           (Rust fills static tp_* C-arrays -> PyType_Ready() wraps them as descriptors
 │            and injects them into Class.__dict__ when m.add_class::<T>() runs)
 │
 ├── 2. "How do two instances (dog=10, cat=15) have different values if 'age' is in Class.__dict__?"
 │      └──► ARCHETYPE B: The Stateless Descriptor + Instance Offset Model (__slots__)
 │           (Class.__dict__['age'] holds a stateless getset_descriptor; calling dog.age
 │            passes dog's pointer 0x1000 into __get__(0x1000) to read dog's own heap block)
 │
 ├── 3. "How does Python read raw binary Rust struct fields (age: i32) across the 16-byte wall?"
 │      └──► ARCHETYPE C: The Hidden C-ABI Trampoline (__pyo3_get_* / __pyo3_set_*)
 │           (Python NEVER reads Rust memory! #[pyo3(get)] generates an extern "C" Rust
 │            function that adds +16..20 bytes to 0x1000, reads i32, and calls PyLong_FromLong)
 │
 └── 4. "How do I pass data between Rust and Python without double-copying my struct?"
        ├── Passing a #[pyclass] instance?
        │     └──► ARCHETYPE D1: Zero-Copy Handle Passing (Py::new -> Py<T> / Bound<'py, T>)
        │          (Allocate once on Python heap; pass &Py<T> to only increment ob_refcnt)
        └── Calling Constructor vs Calling Method?
              └──► ARCHETYPE D2: Instantiation (#[new] / Py::new) vs Invocation (call_method1)
========================================================================================
```

---

## ⚡ 5. Archetype Matrix

| Archetype | Trigger Keywords / Doubt | Core Invariant & Mechanism | Macro / Trait Requirement | Anchor Concept |
| :--- | :--- | :--- | :--- | :--- |
| **A. `PyType_Ready` Bridge** | *"Where are `tp_methods` and `tp_getset` in Python? Why not just `__dict__`?"* | Static `tp_*` C-arrays are compile-time staging tables; `PyType_Ready()` converts them into `Class.__dict__` descriptors at import. | `#[pyclass]` + `#[pymethods]` + `m.add_class::<T>()` | `PyTypeObject` $\rightarrow$ `Class.__dict__` |
| **B. Descriptor Offset Model** | *"If `'age'` is in `Class.__dict__`, how can `dog.age=10` and `cat.age=15` coexist?"* | `Class.__dict__['age']` stores a function wrapper (`getset_descriptor`), **not** the value. The value lives in each instance's `PyCell<T>`. | `#[pyo3(get, set)]` on struct fields (or `#[pyclass(dict)]` for dynamic dict) | CPython `__slots__` & Descriptor Protocol |
| **C. Binary Wall Trampoline** | *"How can Python read raw binary Rust struct memory when Python only knows dicts?"* | Python is blind past Byte 16. Rust generates an `extern "C"` trampoline that jumps the $+16\text{B}$ wall, reads Rust memory, and returns a `PyObject*`. | `#[pyo3(get)]` / `#[pyo3(set)]` / `#[pymethods]` | `PyCell<T>` memory layout & `PyLong_FromLong` |
| **D. Zero-Copy Handle Lifecycle** | *"Why pass `Py<T>` instead of `&T`? How do I avoid double-copying?"* | Python requires a 16-byte `PyObject` header (`0x1000`). `Py<T>` points to that header on the Python heap; passing it only increments `ob_refcnt`. | `Py::new(py, val)`, `Py<T>`, `Bound<'py, T>`, `IntoPyObject` | Reference Counting vs Primitive Conversion |

---

## 📐 6. Formula & Invariant Bank (The "Way")

### Archetype A: The `PyTypeObject` $\rightarrow$ `PyType_Ready()` $\rightarrow$ `Class.__dict__` Equation
* **The Way in Plain English:** Compile-time Rust macros populate static C-arrays (`tp_methods`, `tp_getset`, `tp_new`) inside `PyTypeObject`, and runtime module initialization calls `PyType_Ready()` to wrap every entry into a Python descriptor inside `Class.__dict__`.
* **The Transformation Pipeline:**
  $$\text{Rust } \texttt{\#[pymethods]} \xrightarrow{\text{Compile}} \texttt{PyTypeObject.tp\_*} \xrightarrow{\texttt{PyType\_Ready()}} \texttt{Class.\_\_dict\_\_["name"]} = \text{Descriptor}$$
* **Base Case / Registration Setup:** Until `m.add_class::<Animal>()` (or first instantiation) calls `PyType_Ready()`, `AnimalType.tp_dict` is `NULL`.

### Archetype B: The Multi-Instance Descriptor Resolution Invariant
* **The Way in Plain English:** When looking up `instance.attr`, Python checks `Class.__dict__["attr"]`; if the object found has a `__get__` slot (a descriptor), Python does not return the descriptor itself—it invokes `__get__(descriptor, instance_ptr, Class)` so the descriptor can extract the value from that specific `instance_ptr`.
* **The Pointer Dispatch Formula:**
  $$\texttt{dog.age} \iff \texttt{type(dog).\_\_dict\_\_["age"].\_\_get\_\_}(\texttt{0x1000}, \texttt{Animal}) \implies \text{ returns } 10$$
  $$\texttt{cat.age} \iff \texttt{type(cat).\_\_dict\_\_["age"].\_\_get\_\_}(\texttt{0x2000}, \texttt{Animal}) \implies \text{ returns } 15$$

### Archetype C: The 16-Byte Binary Wall Offset Calculation
* **The Way in Plain English:** Every `#[pyclass]` instance on the Python heap (`PyCell<T>`) starts with a 16-byte CPython header (`ob_refcnt` + `ob_type`) followed by an 8-byte borrow flag; the Rust trampoline adds this compile-time byte offset to `slf: *mut PyObject` to read or mutate the raw Rust struct field.
* **The Trampoline Pointer Arithmetic:**
  $$\text{Rust Field Address} = \underbrace{\texttt{slf\_ptr}}_{\text{Python }\texttt{0x1000}} + \underbrace{\text{sizeof}(\texttt{PyObject Header})}_{16\text{ bytes}} + \underbrace{\text{sizeof}(\texttt{BorrowFlag})}_{8\text{ bytes}} + \text{offsetof}(\texttt{T}, \texttt{field})$$

### Archetype D: Zero-Copy Handle vs Primitive Conversion Rule
* **The Way in Plain English:** Passing a `Py<T>` or `Bound<'py, T>` handle across FFI copies zero struct bytes because the data already lives inside a `PyCell<T>` on the Python heap; reading a primitive field (`i32`) across FFI allocates a new `PyLongObject` because Python cannot hold unboxed 4-byte integers.
* **The FFI Cost Equation:**
  $$\text{Cost}(\text{Pass }\texttt{\&Py<T>}) = \texttt{Py\_INCREF}(\texttt{0x1000}) = O(1) \text{ time, } 0 \text{ bytes copied}$$
  $$\text{Cost}(\text{Read }\texttt{dog.age: i32}) = \text{Read } 4\text{B at offset} + \texttt{PyLong\_FromLong}(10) = 28\text{B Python heap alloc}$$

---

## 🐍 7. Code Templates

### Template 1: Complete `#[pyclass]` Blueprint, Dunders, Getters/Setters & Module Injection
```rust
use pyo3::prelude::*;

// 1. Blueprint: Generates static PyTypeObject (no instance __dict__ by default, like __slots__)
#[pyclass]
pub struct Animal {
    #[pyo3(get, set)] // -> Generates C-ABI trampolines in tp_getset -> Animal.__dict__['age']
    pub age: i32,
}

// 2. Methods & Dunders: Populates tp_new, tp_str, and tp_methods
#[pymethods]
impl Animal {
    #[new] // -> Wires into tp_new -> Animal.__dict__['__new__']
    pub fn new(age: i32) -> Self {
        Animal { age }
    }

    pub fn __str__(&self) -> String { // -> Wires into tp_str -> Animal.__dict__['__str__']
        format!("Animal(age={})", self.age)
    }

    pub fn speak(&self) -> &'static str { // -> Entry in tp_methods -> Animal.__dict__['speak']
        "Roar!"
    }
}

// 3. Module Injection: Runs PyType_Ready() & sets coldwind_core.__dict__['Animal'] = Animal
#[pymodule]
fn coldwind_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Animal>()?;
    Ok(())
}
```

### Template 2: Zero-Copy Instance Allocation (`Py::new`) & Method Invocation from Rust
```rust
use pyo3::prelude::*;

pub fn pass_without_copying(py: Python<'_>, py_callback: &Bound<'_, PyAny>) -> PyResult<()> {
    // 1. Allocate Animal ONCE on the Python heap inside a PyCell (Address 0x1000)
    let dog_handle: Py<Animal> = Py::new(py, Animal { age: 10 })?;

    // 2. Pass handle to Python function -> ZERO struct copy! Only increments ob_refcnt on 0x1000
    py_callback.call1((&dog_handle,))?;

    // 3. Mutate the same 0x1000 instance directly from Rust via PyCell borrow-checking
    dog_handle.bind(py).borrow_mut().age = 12;

    // 4. Invoke a Python-exposed method on the 0x1000 instance
    let sound: String = dog_handle.bind(py).call_method0("speak")?.extract()?;
    assert_eq!(sound, "Roar!");
    Ok(())
}
```

---

## 🔬 8. Python & Rust FFI Memory Reality

This diagram shows the **entire Stage 3.1 architecture in one unified view**: how `Module.__dict__`, `Class.__dict__` (`PyTypeObject`), and two separate `PyCell<Animal>` instances (`dog` at `0x1000` and `cat` at `0x2000`) interact across the 16-byte Binary Wall.

```text
Title: Unified Stage 3.1 Memory Map (Module.__dict__ -> Class.__dict__ -> PyCell Binary Wall)
=============================================================================================
[1. MODULE OBJECT: coldwind_core]
 └── __dict__: { "Animal": ──────────────────────┐
                                                 ▼
[2. CLASS OBJECT (PyTypeObject): Animal] ◄───────┘
 ├── tp_basicsize: 24 bytes (16B Header + 4B BorrowFlag + 4B Rust Animal Struct)
 ├── tp_str: ──────► __pyo3_str_trampoline(slf: *mut PyObject) -> *mut PyObject
 └── tp_dict (Animal.__dict__ — populated by PyType_Ready() from tp_methods & tp_getset):
      ├── "__new__" : <builtin_function_or_method> ──► __pyo3_new_trampoline
      ├── "speak"   : <method_descriptor>          ──► __pyo3_speak_trampoline(slf)
      └── "age"     : <getset_descriptor>          ──► __pyo3_get_age(slf) / __pyo3_set_age(slf, val)
                                                          │
       ┌──────────────────────────────────────────────────┴───────────────────────┐
       │ Calls __pyo3_get_age(0x1000)                      Calls __pyo3_get_age(0x2000)
       ▼                                                                          ▼
[3A. INSTANCE 1: dog (Heap 0x1000)]                        [3B. INSTANCE 2: cat (Heap 0x2000)]
┌─────────────────────────────────────────┐                ┌─────────────────────────────────────────┐
│ 0x1000 [00..08] ob_refcnt = 1           │                │ 0x2000 [00..08] ob_refcnt = 1           │
│ 0x1008 [08..16] ob_type   = &Animal     │                │ 0x2008 [08..16] ob_type   = &Animal     │
╞═════════════════════════════════════════╡                ╞═════════════════════════════════════════╡
│ 🧱 16-BYTE BINARY WALL (Python Blind) 🧱│                │ 🧱 16-BYTE BINARY WALL (Python Blind) 🧱│
╞═════════════════════════════════════════╡                ╞═════════════════════════════════════════╡
│ 0x1010 [16..20] borrow_flag = 0         │                │ 0x2010 [16..20] borrow_flag = 0         │
│ 0x1014 [20..24] age: i32 = 10  ◄──Read! │                │ 0x2014 [20..24] age: i32 = 15  ◄──Read! │
└─────────────────────────────────────────┘                └─────────────────────────────────────────┘
=============================================================================================
```

---

## ⚠️ 9. Trap Archive

1. **Trap 1: Thinking `Class.__dict__['age']` Makes `age` a Shared Class Variable**
   * **Mistake:** Assuming that because `'age'` is in `Animal.__dict__` (and not in `dog.__dict__`), `dog` and `cat` will overwrite each other's `age`.
   * **Why it fails mathematically:** `Animal.__dict__['age']` stores a **stateless function wrapper (`getset_descriptor`)**, not an integer. When `dog.age` is evaluated, Python passes `dog`'s unique heap address (`0x1000`) into `descriptor.__get__(0x1000)`, which reads offset `+20` inside `0x1000`.
   * **10-second recall fix:** *Class `__dict__` holds the key (descriptor function); Instance `PyCell` (`0x1000`) holds the vault (`10`).*

2. **Trap 2: Assuming Python Can Directly Read Rust Struct Binary Memory**
   * **Mistake:** Believing Python's interpreter somehow parses the Rust struct layout past the 16-byte `PyObject` header when `dog.age` is called.
   * **Why it fails mathematically:** Python only knows `PyObject*` headers and dictionaries; dereferencing raw Rust `i32` bytes (`0x0000000A`) as a `PyObject*` would cause an immediate `SIGSEGV`.
   * **10-second recall fix:** *Python never crosses the 16-byte wall—`#[pyo3(get)]` generates a Rust `extern "C"` trampoline that reads the `i32` and calls `PyLong_FromLong`.*

3. **Trap 3: Trying to Use Raw `PyCell<T>` or Raw Pointers Without `#[pyclass]`**
   * **Mistake:** Attempting to allocate a `PyCell<T>` or pass `&T` to Python without deriving `#[pyclass]`.
   * **Why it fails mathematically:** Every Python object (`PyCell<T>`) has an `ob_type` pointer at bytes `8..16` that **must** point to a valid `PyTypeObject` C-struct (`tp_dealloc`, `tp_dict`, `tp_basicsize`). `#[pyclass]` is what generates that static `PyTypeObject`.
   * **10-second recall fix:** *`PyCell<T>` is the envelope; `#[pyclass]` is the `PyTypeObject` stamp required on the envelope.*

4. **Trap 4: Passing Raw Rust Structs Instead of `Py<T>` Handles (The Double-Copy Trap)**
   * **Mistake:** Cloning a Rust struct or converting its fields into a Python dict/list every time you call a Python method, or trying to pass `&Animal` directly to `call1`.
   * **Why it fails mathematically:** Passing a value that converts into a new Python object allocates duplicate heap memory on every call, whereas `&Animal` lacks the 16-byte `PyObject` header.
   * **10-second recall fix:** *Allocate once with `let h: Py<Animal> = Py::new(py, animal)?;` and pass `&h`—zero bytes copied, only `ob_refcnt` incremented.*

5. **Trap 5: Expecting `dog.__dict__` to Exist on a Default `#[pyclass]`**
   * **Mistake:** Running `dog.__dict__` or `dog.new_attr = 99` in Python on a default `#[pyclass]` struct and getting `AttributeError`.
   * **Why it fails mathematically:** By default, `#[pyclass]` sets `tp_dictoffset = 0` (identical to Python's `__slots__`) so instances only consume the exact size of the Rust struct.
   * **10-second recall fix:** *Add `#[pyclass(dict)]` if and only if you need a dynamic per-instance `__dict__` alongside your Rust struct fields.*

---

## 📚 10. Stage 3.1 Concept & Mechanism Library

| # | Stage 3.1 Topic / Doubt | CPython / PyO3 Mechanism | Archetype | 1-Line Mental Model + Key Trap |
| :--- | :--- | :--- | :--- | :--- |
| **1** | Standard Python `__dict__` vs `tp_*` slots | `PyTypeObject` + `PyType_Ready()` | Archetype A | `tp_methods` and `tp_getset` are C staging arrays that `PyType_Ready()` unpacks into `Class.__dict__`. |
| **2** | Why `#[pyclass]` is needed over `PyCell` | `PyTypeInfo` & `ob_type` pointer | Archetype A | `PyCell` is the heap instance container; `#[pyclass]` generates the `PyTypeObject` that `ob_type` points to. |
| **3** | `#[new]` vs `call` / `call_method` | `tp_new` vs `PyObject_CallMethod` | Archetype D | `#[new]` / `Py::new` allocates the `PyCell` on the heap; `call_method` invokes descriptors on an existing `PyCell`. |
| **4** | Dunder methods (`__str__`, `__len__`) | Direct C-VTable slots (`tp_str`, `sq_length`) | Archetype A | Defined in `#[pymethods]`; wired to fast C-struct function pointers AND wrapped into `Class.__dict__`. |
| **5** | Multi-instance state (`dog=10`, `cat=15`) | `getset_descriptor` + instance pointer | Archetype B | `Class.__dict__['age']` holds the getter function; `10` and `15` live in `dog` (`0x1000`) and `cat` (`0x2000`). |
| **6** | Reading Rust binary fields from Python | Hidden `extern "C"` Rust Trampolines | Archetype C | Python passes `0x1000` to Rust's trampoline; Rust jumps $+16\text{B}$, reads `i32`, and calls `PyLong_FromLong`. |
| **7** | `call_method` argument trait & zero-copy | `IntoPy<PyObject>` / `ToPyObject` & `Py<T>` | Archetype D | Primitives allocate a new `PyObject`; passing `&Py<T>` or `&Bound<'py, T>` is zero-copy (`Py_INCREF` only). |

---

## 📎 11. Variation Notes & Reports Directory

* **[Report 01: `#[pyclass]`, `PyTypeObject`, and the CPython Descriptor Bridge](../reports/pyo3-internals/01-pyclass-pytypeobject-and-descriptor-bridge.md)** — Full breakdown of the 1-to-1 mapping between Standard Python and PyO3, the `PyType_Ready()` engine, `PyCell` vs `#[pyclass]`, and all macro attributes (`#[new]`, `#[pyo3(get, set)]`, `#[pyclass(dict)]`).
* **[Report 02: The Binary Wall, C-ABI Trampolines, and PyO3's FFI Memory Model](../reports/pyo3-internals/02-binary-wall-trampolines-and-memory-model.md)** — Deep-dive investigation into how `dog` (`0x1000`) and `cat` (`0x2000`) maintain separate binary state via descriptors, the exact Rust trampoline code that crosses the 16-byte `PyObject` header wall, and zero-copy `Py<T>` handle passing vs primitive conversion.
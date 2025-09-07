# Repository Reorganization Plan

**Separating the Systematic Engineering Framework from RumbleDome**

---

## Current State
The systematic engineering methodology has grown organically within the RumbleDome project, creating a mixed repository where:
- Framework code is in `tools/systematic_engineering_core.py`  
- Methodology documentation is in `docs/SYSTEMATIC_ENGINEERING_*.md`
- RumbleDome-specific implementation uses the framework
- Everything is tightly coupled

## Target State
Clean separation where:
- **Systematic Engineering Framework** is a standalone, reusable methodology
- **RumbleDome** is a flagship example implementation  
- **Other domains** can easily adopt the framework
- **Documentation** clearly separates methodology from domain examples

---

## Recommended Structure

### New Repository: `systematic-engineering-framework`

```
systematic-engineering-framework/
├── README.md                           # Framework overview and quick start
├── LICENSE                             # MIT or Apache 2.0
├── 
├── framework/
│   ├── systematic_engineering_core.py  # Core framework
│   ├── templates/                      # Base templates
│   │   ├── hal_implementation.py
│   │   ├── control_system.py
│   │   └── safety_system.py
│   └── validators/                     # Validation modules
│       ├── traceability.py
│       ├── cross_reference.py
│       └── domain_specific.py
│
├── docs/
│   ├── README.md                       # Documentation index
│   ├── METHODOLOGY_OVERVIEW.md         # What is systematic engineering
│   ├── QUICKSTART_GUIDE.md            # 30-minute adoption guide
│   ├── TECHNICAL_IMPLEMENTATION.md     # Framework internals
│   ├── CONFIGURATION_REFERENCE.md      # Complete config schema
│   ├── TEMPLATE_DEVELOPMENT.md         # How to create custom templates
│   ├── LEGACY_ADOPTION.md              # Working with existing projects
│   └── CASE_STUDIES.md                 # Success stories and lessons learned
│
├── examples/
│   ├── rumbledome/                     # Automotive boost controller
│   │   ├── project-config.json
│   │   ├── rumbledome-cli
│   │   ├── docs/                       # RumbleDome-specific docs
│   │   └── generated/                  # Example generated code
│   ├── flight-controller/              # Aerospace example
│   │   ├── project-config.json
│   │   ├── aero-cli
│   │   └── docs/
│   ├── web-api/                        # Software system example
│   │   ├── project-config.json
│   │   └── docs/
│   └── iot-device/                     # IoT/embedded example
│       ├── project-config.json
│       └── docs/
│
├── tools/
│   ├── framework-cli                   # Generic CLI
│   ├── project-template-generator      # Scaffolding tool
│   ├── legacy-analyzer                 # Code archaeology tools
│   └── validation-dashboard            # Health metrics visualization
│
└── tests/
    ├── framework/                      # Framework unit tests
    ├── integration/                    # End-to-end tests
    └── examples/                       # Example project tests
```

### Updated RumbleDome Repository

```
rumbledome/
├── README.md                           # RumbleDome-specific readme
├── LICENSE                             # Original license
├── .gitmodules                         # Git submodule config
├── 
├── systematic-engineering/             # Git submodule
│   └── [systematic-engineering-framework repo]
│
├── project-config.json                 # RumbleDome configuration
├── rumbledome-cli                      # RumbleDome CLI wrapper
│
├── docs/                               # RumbleDome domain documentation
│   ├── README.md                       # RumbleDome docs index
│   ├── Context.md                      # Original RumbleDome docs
│   ├── Requirements.md
│   ├── Safety.md
│   ├── TechnicalSpecs.md
│   ├── Architecture.md
│   ├── Hardware.md
│   ├── Implementation.md
│   └── [other RumbleDome-specific docs]
│
├── crates/                             # Original Rust workspace
│   ├── rumbledome-core/
│   ├── rumbledome-hal/
│   ├── rumbledome-fw/
│   └── [other crates]
│
├── kicad/                              # Hardware design files
├── tools/                              # RumbleDome-specific tools
└── generated/                          # Framework-generated code
```

---

## Migration Steps

### Phase 1: Extract Framework
1. **Create new repository**: `systematic-engineering-framework`
2. **Move core files**:
   - `tools/systematic_engineering_core.py` → `framework/systematic_engineering_core.py`
   - Methodology docs → `docs/`
   - Generic templates → `framework/templates/`
3. **Create example structure**:
   - Move RumbleDome config to `examples/rumbledome/`
   - Create other domain examples
4. **Update documentation**:
   - Framework-focused README
   - Clear separation of methodology vs. domain docs

### Phase 2: Update RumbleDome
1. **Add framework as dependency**:
   - Git submodule or package reference
   - Update CLI to use external framework
2. **Clean up mixed content**:
   - Remove methodology docs from RumbleDome repo
   - Keep only domain-specific documentation
   - Update cross-references

### Phase 3: Enhance Framework
1. **Improve standalone usability**:
   - Better project scaffolding tools
   - Enhanced documentation
   - More example domains
2. **Add framework-specific features**:
   - Web-based configuration editor
   - Validation dashboard
   - Template marketplace

---

## Benefits of This Structure

### For Framework Adoption
- **Clear entry point** for new users
- **Multiple domain examples** showing versatility  
- **Clean separation** of methodology from specific implementations
- **Easy to fork** for custom domains

### For RumbleDome Development
- **Focused repository** on boost controller domain
- **Framework updates** propagate automatically
- **Clear documentation** separation
- **Example status** showcases methodology value

### For Portfolio/Publishing
- **Standalone methodology** that stands on its own merits
- **RumbleDome as flagship example** of successful application
- **Multiple domains** demonstrate portability
- **Professional presentation** suitable for public release

---

## Implementation Priority

### Immediate (This Week)
- [ ] Create `systematic-engineering-framework` repository structure
- [ ] Move framework code and methodology documentation  
- [ ] Create basic README and quickstart guide
- [ ] Test framework extraction works independently

### Short-term (Next 2 Weeks)  
- [ ] Update RumbleDome to use extracted framework
- [ ] Create additional domain examples (flight controller, web API)
- [ ] Enhance framework documentation
- [ ] Add project scaffolding tools

### Long-term (Next Month)
- [ ] Public release preparation
- [ ] Community documentation and contribution guides
- [ ] Framework enhancement based on multi-domain usage
- [ ] Integration tooling (CI/CD, IDE plugins)

---

## File Movement Checklist

### Framework Repository Files
- [x] `tools/systematic_engineering_core.py`
- [x] `docs/SYSTEMATIC_ENGINEERING_PLATFORM.md`
- [x] `docs/QUICKSTART_GUIDE.md`  
- [x] `docs/TECHNICAL_IMPLEMENTATION.md`
- [x] `docs/METHODOLOGY_TRANSFORMATION.md`
- [x] `docs/LEGACY_PROJECT_ADOPTION.md`
- [x] `docs/DOCUMENTATION_INDEX.md`
- [x] `tools/project-config.json` (as template)
- [x] `tools/example-flight-controller-config.json`

### RumbleDome-Specific Files (Stay)
- [x] All `crates/` content
- [x] `docs/Context.md`, `Requirements.md`, etc.
- [x] `kicad/` hardware files  
- [x] Domain-specific tooling
- [x] Build configuration
- [x] Project-specific documentation

### Hybrid Files (Need Separation)
- [x] `tools/cli` → Split into framework CLI + RumbleDome wrapper
- [x] Mixed documentation → Separate methodology from domain
- [x] Configuration files → Framework templates + project config

This reorganization preserves all the work while creating a clean, publishable framework that others can adopt.
# Traceability ID Management Process

## Current State
- **101 traceability IDs** across 19 documentation files
- **Systematic engineering validation** prevents commits with duplicate IDs
- **Manual ID assignment** prone to conflicts during extensive documentation evolution

## Process Improvements

### 1. ID Allocation Protocol

**Before creating new traceability IDs:**
```bash
# Check next available ID (RECOMMENDED)
./tools/cli id-check T2-CONTROL

# Alternative: Manual scan
./tools/cli report --export temp.md
grep "T2-CONTROL-" docs/*.md | sort
```

**ID Assignment Rules:**
- Use sequential numbering within categories
- Leave gaps for future expansion (multiples of 5)
- Never reuse existing IDs for different concepts
- Update all cross-references when renumbering

### 2. Categories and Ranges

**Current Allocations (as of checkpoint):**
- T2-CONTROL-001 through T2-CONTROL-022: ALLOCATED
- Next available determined by: `./tools/cli id-check T2-CONTROL`

**Recommended Category Ranges:**
- Core control algorithms: T2-CONTROL series (001-999)
- Safety systems: T2-SAFETY series (001-999)
- Hardware interfaces: T2-HAL series (001-999) 
- Diagnostics: T2-DIAGNOSTICS series (001-999)

### 3. AI Collaboration Protocol

**When Claude needs new traceability IDs:**
```bash
# Step 1: Check next available
./tools/cli id-check T2-CONTROL

# Step 2: Allocate with automatic registry
./tools/cli id-allocate T2-CONTROL "Descriptive Title"

# Step 3: Verify before committing
./tools/cli validate
```

**Manual Process (if CLI unavailable):**
1. Check existing allocations first
2. Use next sequential number in appropriate category
3. Update cross-references for any renumbering
4. Verify systematic engineering compliance

**Template for new ID creation:**
```markdown
🔗 T2-CATEGORY-XXX: Descriptive Title
Derived From: Parent requirements/decisions
Decision Type: Engineering Decision/Direct Derivation/etc.
Engineering Rationale: Why this approach was chosen
AI Traceability: What code/algorithms this drives
```

### 4. Refactoring Safety

**When moving or splitting content:**
- Create new IDs rather than reusing existing ones
- Update all references to changed IDs
- Run validation before and after changes
- Maintain conceptual integrity of traceability chains

### 5. Quality Gates

**Pre-Commit Validation:**
- Systematic engineering validation blocks duplicate IDs ✅
- Manual review of traceability integrity for major changes
- Cross-reference validation for renamed/moved IDs

**Documentation Updates:**
- This process document updated whenever ID management evolves
- Examples added for common ID allocation scenarios
- Tool enhancement recommendations captured for future implementation

## Enhanced CLI Tools ✅ IMPLEMENTED

**Available ID Management Commands:**
```bash
./tools/cli id-check T2-CONTROL              # Find next available ID
./tools/cli id-allocate T2-CONTROL "Title"   # Auto-assign and register
./tools/cli id-trace T2-CONTROL-013          # Show all references with context
./tools/cli id-validate                      # Suggest fixes for issues
./tools/cli help                             # Comprehensive command help
```

**Registry Management:**
- Automated registry generation from document scanning
- ID reservation system for work-in-progress
- Cross-reference validation during refactoring
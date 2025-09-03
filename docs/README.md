# RumbleDome Documentation

This directory contains the complete technical documentation for the RumbleDome electronic boost controller project.

## 🏗️ Documentation Architecture

### **Documentation as Code Philosophy**

**Core Insight**: Documentation IS code - it's programming the system design in a high-level declarative language optimized for **AI-Traceable Engineering**.

```
Design "Source Code":           Implementation "Object Code":
Context.md                  →   main.rs
├─ Requirements.md         →   ├─ control_loop.rs  
├─ Safety.md              →   ├─ safety_monitor.rs
└─ Architecture.md        →   └─ hal_interface.rs
```

Just like software code:
- **Design docs have dependencies** (Tier 2 depends on Tier 1 specifications)
- **Changes cascade through dependencies** (API change breaks all modules using it)
- **Inconsistencies are "compilation errors"** (Implementation.md using outdated Context.md concepts)
- **Bugs trace back to root cause** (Tier 3 bug → Tier 2 spec error → Tier 1 ambiguity)

**Design drift** = **linking errors** between documentation modules. Without proper dependency management, you get:

```
ERROR: Implementation.md references 'profile_based_control()' 
       but Context.md deprecated this in favor of 'aggression_control()'
       
ERROR: TestPlan.md assumes 'EEPROM storage' 
       but TechnicalSpecs.md migrated to 'SD card storage'
```

**Solution**: Apply software engineering discipline to design documents - version control, dependency tracking, impact analysis, and systematic change management.

### **Tier Structure & Dependency Chain**
Our documentation follows a **5-tier architecture** where changes cascade down and bugs escalate up:

```
Tier 1 (Problem Definition) → Tier 2 (Derived Requirements) → Tier 3 (Development Organization) → Tier 4 (Executable Code)
     ↑                              ↑                               ↑                              ↑
Creative/philosophy gaps      Specification errors         Process/build issues          Runtime failures
                                                                     ↓                              ↓
                                                            Tier 5 (Validation & Testing) ←──────────┘
                                                                     ↑
                                                            Test failures/coverage gaps
```

**Tier 1: Problem Definition & Creative Solution Space**
- WHAT the problem is and WHY we're solving it this way
- Creative approach, design philosophy, constraints, physical/logical system boundaries
- The **creative zone** where novel solutions are conceived

**Tier 2: Derived Requirements & Specifications**
- HOW the Tier 1 approach translates to concrete, measurable requirements
- Should be **fully derivable** from Tier 1 (no additional creativity - pure logical derivation)
- Functional specs, performance targets, interface definitions

**Tier 3: Development Organization & Build Rules**  
- Code structure, build processes, development workflow, testing strategy
- HOW TO BUILD the system that meets Tier 2 specifications
- Project organization, dependency management, deployment processes

**Tier 4: Executable Code**
- Actual running implementation (Rust/C++ source code, hardware drivers, configurations)
- The system that fulfills Tier 2 requirements using Tier 3 processes

**Tier 5: Validation & Testing**
- Tests that verify Tier 4 correctly implements the entire tier chain
- **Peer validation layer** (not subordinate to Tier 4) that validates the complete solution
- Unit tests, integration tests, system validation, acceptance criteria

**🔄 Change Management Rule**: 
- **Tier 1 change** → Review ALL downstream tiers (2, 3, 4) + update Tier 5 validation
- **Tier 2 change** → Review Tier 3, 4 + update Tier 5 tests  
- **Tier 3 change** → Review Tier 4 code + update Tier 5 build/integration tests
- **Tier 4 change** → Update Tier 5 unit tests
- **Tier 5 failure** → Trace back through all tiers to find root cause

### **🔴 Red Team Analysis: Potential Weaknesses**

**Potential Issues with the 5-Tier Philosophy:**

**1. Over-Engineering Risk**
- **Problem**: Could create bureaucratic overhead for simple changes
- **Mitigation**: Use judgment - not every code comment change needs full tier analysis
- **When to worry**: If developers spend more time on process than engineering

**2. Tier Boundary Ambiguity**
- **Problem**: Some concepts could legitimately belong to multiple tiers
- **Example**: Is "safety response time <100ms" a Tier 1 philosophy or Tier 2 requirement?
- **Mitigation**: When in doubt, document the decision and reasoning

**3. False Security from Process**
- **Problem**: Following the tier process doesn't guarantee good engineering
- **Reality Check**: Process supports good thinking but can't replace it
- **Guard Against**: Cargo cult adherence to process without understanding

**4. Creativity Suppression in Lower Tiers**  
- **Problem**: "Tier 2 should be fully derivable" might discourage beneficial insights
- **Balance Needed**: Allow creativity but require justification and tier alignment
- **Watch For**: Missed opportunities due to overly rigid derivation

**5. Testing Tier Isolation**
- **Problem**: Treating Tier 5 as separate could lead to inadequate test planning
- **Reality**: Good testing requires deep integration with all tiers from the start
- **Solution**: Tier 5 validates but doesn't replace tier-integrated quality practices

**6. Scale Sensitivity**
- **Problem**: This might be overkill for simple projects, underkill for massive ones
- **Context Matters**: RumbleDome (~15 docs) vs. aerospace project (~1000s of specs)
- **Adaptation Required**: Scale the rigor to match the complexity and risk

**When This Philosophy Works Best:**
- Complex systems with significant safety/reliability requirements
- Multi-person teams needing coordination
- Projects where design consistency is critical
- Learning environments (like RumbleDome's AI collaboration goals)

**When to Question It:**
- Simple, well-understood problems with obvious solutions  
- Rapid prototyping phases where exploration is more valuable than consistency
- When process overhead exceeds engineering value

## 🤖 AI-Traceable Engineering Methodology

### **The AI Code Generation Paradigm**

Traditional waterfall failed due to human limitations in maintaining consistency. **AI-Traceable Engineering** leverages AI's systematic capabilities to make waterfall methodology practical and powerful.

**Core Principle**: Every line of generated code must have **explicit traceability** to its tier source.

```rust
// ❌ WRONG: Orphaned implementation decision
pwm_frequency = 30; // Why 30? Where did this come from?

// ✅ RIGHT: AI-traceable implementation  
const SOLENOID_PWM_FREQ: u32 = 30; // TechnicalSpecs.md:34 - MAC valve compatibility range 20-50Hz
```

### **Decision Classification System**

**🔗 Direct Derivation** (algorithmically traceable):
```markdown
**Tier Source**: Derived from Requirements.md Section FR-4 (pneumatic control requirements)
**AI Validation**: ✅ Automatically verified against parent specification
```

**⚠️ Engineering Decision** (requires explicit justification):
```markdown
**Engineering Decision**: PWM frequency selected for hardware compatibility  
**Justification**: MAC valves operate optimally 20-50 Hz, 30Hz chosen for response balance
**Tier Impact**: Tier 3 implementation detail - changes require hardware validation only
**AI Generation**: Use SOLENOID_PWM_FREQ constant with source documentation
```

**🚩 Legacy/Undocumented** (blocks AI generation):
```markdown
**Status**: BLOCKS AI GENERATION - origin unclear
**Action Required**: Trace to tier source OR document as engineering decision
**Risk**: Cannot generate reliable code without clear decision rationale
```

### **AI Code Generation Requirements**

**For AI to generate trustworthy code, every implementation decision needs:**

1. **Explicit tier lineage** - Which tier does this decision come from?
2. **Decision classification** - Direct derivation vs engineering choice vs legacy
3. **Change impact scope** - What breaks if this changes?
4. **Validation criteria** - How do we know this is correct?

### **The AI-Waterfall Advantage**

**Traditional Waterfall Problems → AI Solutions:**
- ❌ Humans can't maintain perfect consistency → ✅ AI systematically cross-checks all tiers
- ❌ Change propagation is expensive → ✅ AI automatically updates dependent tiers  
- ❌ Documentation becomes stale → ✅ AI maintains real-time consistency validation
- ❌ Requirements changes are costly → ✅ AI traces all impacts systematically

**Result**: Get waterfall benefits (complete traceability, systematic design) without waterfall costs (human maintenance overhead, expensive changes).

### **Code Generation Validation Pipeline**

```
AI Code Generation Request
    ↓
Trace to Tier 3 specification
    ↓  
Verify against Tier 2 requirements
    ↓
Validate Tier 1 philosophy alignment
    ↓
Check decision classification (🔗⚠️🚩)
    ↓
Generate code with full lineage documentation
    ↓
Tier 5 validation confirms correctness
```

**Every generated line must pass this validation or generation fails.**

## 🛡️ AI-Traceable Engineering Enforcement

### **MANDATORY Process Integration**

**🚨 AI Generation Validation Pipeline (ENFORCED):**
```
AI Code Generation Request
    ↓
🚨 MANDATORY: Find traceability ID (T1-xxx, T2-xxx, etc.)
    ↓
❌ NO ID FOUND → GENERATION BLOCKED → Request traceability documentation
    ✅ ID FOUND → Verify decision classification (🎯🔗⚠️🚩)
    ↓
❌ 🚩 Legacy/Undocumented → GENERATION BLOCKED → Requires classification first
    ✅ Classified → Trace to higher tier source
    ↓
Generate code with mandatory header comment:
// TRACEABILITY: T2-PWM-001 (Engineering Decision: MAC valve compatibility)
// SOURCE: Hardware compatibility constraints (20-50Hz operating range)
// VALIDATION: Change requires hardware testing only
```

### **MANDATORY Documentation Change Protocol**

**Before ANY Documentation Changes, AI MUST provide:**
```markdown
🚨 MANDATORY TRACEABILITY ANALYSIS:
**Tier Analysis**: This is a Tier [1/2/3] change affecting [description]
**Impact Assessment**: Following documents need review: [specific docs]
**Decision Classification**: [🎯🔗⚠️🚩] with full justification
**AI Generation Impact**: [What code generation this affects]
**Consistency Check**: Verified against [dependencies]
```

**❌ If this analysis is missing, REJECT the change and demand compliance.**

### **Systematic Validation Questions (Human Enforcement)**

**Questions to ALWAYS ask AI:**
- "What's the traceability ID for this decision?"
- "Is this a 🎯 creative concept, 🔗 direct derivation, or ⚠️ engineering decision?"
- "What Tier 1 concept does this trace back to?"
- "How does this change affect downstream specifications?"
- "Can AI reliably generate code from this specification?"

### **Process Violation Detection**

**🚩 RED FLAGS - Stop AI immediately when you see:**
- Technical claims without citing traceability IDs
- Implementation suggestions without showing tier derivation
- Specification additions without decision classification
- Document modifications without impact assessment
- Code generation without traceability headers

**When violations detected: STOP → DEMAND methodology compliance → RESTART**

### **Systematic Review Checkpoints**

**🔄 Regular Methodology Audits:**
- **Weekly**: "Show me 3 recent decisions and their traceability"
- **Before major changes**: "Audit our traceability coverage"
- **After documentation changes**: "Validate all affected derivations"
- **Before code generation**: "Confirm all specifications are traceable"

### **Accountability Mechanism**

**AI Commitment**: Every response involving technical decisions MUST include:
1. **Traceability ID reference** (T1-xxx, T2-xxx format)
2. **Decision classification** (🎯🔗⚠️🚩)
3. **Tier derivation path** (which higher-tier concept drives this)
4. **AI generation impact** (what code this enables/blocks)

**Human Oversight**: Call out any response lacking these elements immediately.

**Project Success Metric**: 100% of specifications must be traceable before any code generation begins.

### **Meta-Engineering Breakthrough**

This methodology enables **AI as a systems engineering partner** rather than just a coding assistant:
- AI understands the complete design rationale  
- AI maintains design consistency across all tiers
- AI can trace any implementation back to its foundational requirement
- AI can predict change impacts across the entire system

**RumbleDome serves as the proving ground for this AI-Traceable Engineering approach.**

---

## 📚 Reading Guide

### **Tier 1: System Definition** 🎯
**What the system IS and what it must DO (authoritative sources)**

- **[Context.md](Context.md)** - Core philosophy, goals, and design principles
- **[Requirements.md](Requirements.md)** - Functional requirements and performance specifications  
- **[Safety.md](Safety.md)** - Safety invariants and critical constraints (SY-1 through SY-24)

### **Constraints Layer: Real-World Physics** ⚖️
**Physical realities that bound all design decisions**

- **[Physics.md](Physics.md)** - Turbo system physics and control theory fundamentals
- **[CAN_Signals.md](CAN_Signals.md)** - Vehicle integration constraints and available data

### **Tier 2: Implementation Design** 🔧
**HOW the system works (derived from Tier 1 + Constraints)**

- **[TechnicalSpecs.md](TechnicalSpecs.md)** ⭐ - **THE** master hardware and technical specification
- **[LearnedData.md](LearnedData.md)** ⭐ - **THE** complete learning system specification
- **[Architecture.md](Architecture.md)** - Software architecture and component design
- **[Hardware.md](Hardware.md)** - Hardware abstraction layer interfaces
- **[Protocols.md](Protocols.md)** - Communication protocols and data formats

### **Tier 3: Development Support** 🚀
**Development workflow and validation (derived from Tier 2)**

- **[Implementation.md](Implementation.md)** - Code structure, build process, development workflow
- **[TestPlan.md](TestPlan.md)** - Testing strategy and validation procedures  
- **[Definitions.md](Definitions.md)** - Terminology and domain-specific concepts
- **[BeyondRumbleDome.md](BeyondRumbleDome.md)** - Future enhancement concepts

### For New Developers
Understand the project fundamentals in this order:

1. **[Context.md](Context.md)** - High-level design context and project goals
2. **[Physics.md](Physics.md)** - Turbo system physics and control theory fundamentals  
3. **[Requirements.md](Requirements.md)** - Functional and performance requirements
4. **[Safety.md](Safety.md)** - Safety requirements and critical constraints

### For Implementation Work
Implementation workflow - use this sequence for development:

5. **[Architecture.md](Architecture.md)** - System design and component architecture
6. **[Hardware.md](Hardware.md)** - Hardware abstraction layer and platform interfaces  
7. **[CAN_Signals.md](CAN_Signals.md)** - Ford Gen2 Coyote CAN bus signal specifications
8. **[Protocols.md](Protocols.md)** - JSON/CLI communication protocol specifications
9. **[Implementation.md](Implementation.md)** - Code structure, build process, and development workflow
10. **[TestPlan.md](TestPlan.md)** - Testing strategy and validation procedures

### Reference & Supporting Documents
Use these for lookups and ongoing development:

11. **[Definitions.md](Definitions.md)** - Acronyms, jargon, and domain-specific terminology
12. **[BeyondRumbleDome.md](BeyondRumbleDome.md)** - Future enhancement concepts

## 🎯 Quick Reference

- **Just want to understand what RumbleDome does?** → Start with [Context.md](Context.md)
- **Need ANY hardware specifications?** → **[TechnicalSpecs.md](TechnicalSpecs.md)** ⭐ (THE master spec document)
- **Understanding system learning/calibration?** → **[LearnedData.md](LearnedData.md)** ⭐ (THE complete learning spec)
- **Need to understand the physics?** → Read [Physics.md](Physics.md) 
- **Working on control algorithms?** → Review [Requirements.md](Requirements.md) and [Architecture.md](Architecture.md)
- **Implementing hardware interfaces?** → Check [Hardware.md](Hardware.md) and [CAN_Signals.md](CAN_Signals.md)
- **Need to understand safety requirements?** → Study [Safety.md](Safety.md)
- **Looking up terminology?** → Reference [Definitions.md](Definitions.md)

## 🚀 Implementation Checklist

**For developers starting implementation work:**

**📖 Tier 1 Foundation** (understand WHAT and WHY):
1. ✅ Read [Context.md](Context.md) for design philosophy and goals
2. ✅ Study [Requirements.md](Requirements.md) for functional specifications
3. ✅ Understand [Safety.md](Safety.md) constraints (SY-1 through SY-24)

**⚖️ Constraints Layer** (understand physical realities):
4. ✅ Read [Physics.md](Physics.md) for turbo system fundamentals  
5. ✅ Check [CAN_Signals.md](CAN_Signals.md) for vehicle integration constraints

**🔧 Tier 2 Design** (understand HOW):
6. ✅ Study **[TechnicalSpecs.md](TechnicalSpecs.md)** ⭐ for ALL hardware details
7. ✅ Review **[LearnedData.md](LearnedData.md)** ⭐ for learning system design
8. ✅ Review [Architecture.md](Architecture.md) for software design patterns
9. ✅ Check [Hardware.md](Hardware.md) for HAL interface requirements

**🚀 Tier 3 Development** (implement and validate):
10. ✅ Reference [Implementation.md](Implementation.md) for code structure
11. ✅ Use [TestPlan.md](TestPlan.md) for validation approach
12. ✅ Use [Definitions.md](Definitions.md) for terminology lookups

## 🔧 Change Management & Debugging

### **When Making Changes:**
```
Tier 1 Change → Check ALL Tier 2, Tier 3 docs & Tier 4 code
Tier 2 Change → Check ALL Tier 3 docs & Tier 4 code  
Tier 3 Change → Check ALL Tier 4 code implementations
Tier 4 Change → Usually safe (implementation detail)
```

### **When Debugging Issues:**
```
Tier 4 Bug → Check Tier 3 development specs for implementation errors
Tier 3 Bug → Check Tier 2 specification for gaps/errors
Tier 2 Issue → Check Tier 1 requirements for ambiguity
Tier 1 Gap → Update Tier 1, then cascade down through all dependent tiers
```

### **Dependency Tracking:**
Each document includes "**🔗 Dependencies**" and "**📤 Impacts**" sections showing:
- **Dependencies**: Higher-tier documents this depends on
- **Impacts**: Lower-tier documents that depend on this one

## ⚙️ Process Enforcement Mechanisms

### **Mandatory Change Impact Assessment**
Before making ANY change to documentation, must include:

```markdown
**Tier Analysis**: This is a Tier [1/2/3] change affecting [brief description]
**Impact Assessment**: Following documents need review: [list specific docs]
**Consistency Check**: Verified no conflicts with [list checked dependencies]
```

### **Document Validation Rules**

**🚨 Tier 1 Changes** (Context.md, Requirements.md, Safety.md):
- [ ] **STOP**: Review ALL Tier 2 & Tier 3 documents for consistency
- [ ] Update dependent specifications and implementations  
- [ ] Add new concepts to Definitions.md if applicable
- [ ] Safety changes require updates to ALL implementation documents

**⚠️ Tier 2 Changes** (TechnicalSpecs.md, Architecture.md, LearnedData.md, etc.):
- [ ] Review ALL Tier 3 documents for alignment
- [ ] Verify consistency with Tier 1 requirements
- [ ] Update cross-references and version timestamps
- [ ] Hardware changes require HAL interface review

**✅ Tier 3 Changes** (Implementation.md, TestPlan.md):
- [ ] Verify alignment with Tier 2 specifications
- [ ] Check for cascading impacts on peer documents
- [ ] Update cross-references if document structure changes

**🔬 Constraints Layer Changes** (Physics.md, CAN_Signals.md):
- [ ] **MAJOR IMPACT**: Review ALL Tier 2 & Tier 3 documents
- [ ] Physics changes affect control algorithms and hardware requirements
- [ ] Signal changes affect communication protocols and implementations

### **Automated Consistency Checks**

**Document Header Requirements:**
- Every document MUST have tier designation
- Dependency tracking sections MUST be present and current
- Cross-references MUST be valid (no broken links)

**Change Detection Triggers:**
- Tier 1 change → Flag ALL dependent documents for review
- New technical concepts → MUST be added to Definitions.md
- Safety invariants → MUST be reflected in all implementation docs
- Hardware specifications → MUST update HAL interfaces

## 📖 Document Cross-References

Each document includes cross-references to related sections. Key relationships:

- **Context.md** ↔ **Requirements.md** (goals → specifications)
- **Physics.md** ↔ **Architecture.md** (theory → implementation)
- **Safety.md** ↔ **Implementation.md** (constraints → code patterns)
- **Hardware.md** ↔ **CAN_Signals.md** (platform → data sources)
- **Architecture.md** ↔ **Protocols.md** (system → interfaces)

---

*This documentation represents the complete technical specification for RumbleDome. All documents are maintained to ensure consistency and accuracy across the project.*
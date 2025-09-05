# RumbleDome Documentation

This directory contains the complete technical documentation for the RumbleDome electronic boost controller project.

## 🎯 Quick Start

- **Just want to understand what RumbleDome does?** → Start with [Context.md](Context.md)
- **Need ANY hardware specifications?** → **[TechnicalSpecs.md](TechnicalSpecs.md)** ⭐ (THE master spec document)
- **Understanding system learning/calibration?** → **[LearnedData.md](LearnedData.md)** ⭐ (THE complete learning spec)
- **Need to understand the physics?** → Read [Physics.md](Physics.md) 
- **Working on control algorithms?** → Review [Requirements.md](Requirements.md) and [Architecture.md](Architecture.md)
- **Implementing hardware interfaces?** → Check [Hardware.md](Hardware.md) and [CAN_Signals.md](CAN_Signals.md)
- **Need to understand safety requirements?** → Study [Safety.md](Safety.md)
- **Looking up terminology?** → Reference [Definitions.md](Definitions.md)

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
- **[AI_Collaborative_Engineering.md](AI_Collaborative_Engineering.md)** - Engineering process methodology and documentation standards
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

## 🚀 Implementation Checklist

**For developers starting implementation work:**

**📖 Tier 1 Foundation** (understand WHAT and WHY):
1. ✅ Read [Context.md](Context.md) for design philosophy and goals
2. ✅ Study [Requirements.md](Requirements.md) for functional specifications
3. ✅ Understand [Safety.md](Safety.md) constraints (SY-1 through SY-24)

**⚖️ Constraints Layer** (understand physical realities):
4. ✅ Read [Physics.md](Physics.md) for turbo system fundamentals  
5. ✅ Check [CAN_Signals.md](CAN_Signals.md) for vehicle integration constraints

**🔧 Tier 2 Implementation Design** (understand HOW):
6. ✅ Review [Architecture.md](Architecture.md) for system architecture
7. ✅ Study [LearnedData.md](LearnedData.md) for complete learning system specification
8. ✅ Check [TechnicalSpecs.md](TechnicalSpecs.md) for hardware/platform requirements
9. ✅ Review [Hardware.md](Hardware.md) for HAL interfaces and pin assignments
10. ✅ Study [Protocols.md](Protocols.md) for communication interfaces

**🚀 Tier 3 Development** (BUILD and TEST):
11. ✅ Follow [Implementation.md](Implementation.md) for code structure and build process
12. ✅ Execute [TestPlan.md](TestPlan.md) for validation procedures
13. ✅ Reference [Definitions.md](Definitions.md) for terminology consistency

## 🔧 Change Management

### **When Making Changes:**
1. **Identify the tier** of your change (T1 = philosophy, T2 = specifications, T3 = implementation)
2. **Check dependencies** - what other documents reference the sections you're changing?
3. **Update downstream dependencies** - cascade changes through the tier system
4. **Validate consistency** - ensure no conflicting specifications remain

### **When Debugging Issues:**
1. **Start with Tier 1** - is the problem a missing requirement or unclear philosophy?
2. **Check Tier 2** - are the specifications complete and consistent?
3. **Review Tier 3** - are the implementation details following the specifications?
4. **Trace dependencies** - follow the impact chain from problem back to root cause

## 📋 Documentation Standards

### **Tier Structure**
Our documentation follows a **3-tier architecture** where changes cascade down and bugs escalate up:

```
Tier 1 (Problem Definition) → Tier 2 (Design Specifications) → Tier 3 (Development Support)
     ↑                              ↑                               ↑
Creative/philosophy gaps      Specification errors         Implementation issues
```

### **Decision Traceability**
Each technical decision is marked with:
- **🎯 Core Principle** - Foundational design philosophy
- **🔗 Direct Derivation** - Logically derived from higher-tier decisions  
- **⚠️ Engineering Decision** - Requires engineering judgment and domain expertise

### **AI Collaboration**
For details on our AI-assisted engineering methodology, see:
- **[AI_Philosophy.md](AI_Philosophy.md)** - AI collaboration philosophy and human/AI roles
- **[AI_Collaborative_Engineering.md](AI_Collaborative_Engineering.md)** - Engineering process methodology and documentation standards
- **[SystematicEngineeringTool.md](SystematicEngineeringTool.md)** - CLI tool for validation and consistency checking

## 📖 Document Cross-References

**Context.md** references: Requirements.md, Safety.md, Physics.md  
**Requirements.md** references: Context.md, Safety.md, Architecture.md  
**Architecture.md** references: Requirements.md, Hardware.md, Physics.md  
**Hardware.md** references: TechnicalSpecs.md, Implementation.md  
**Implementation.md** references: Architecture.md, Hardware.md, TestPlan.md  
**TestPlan.md** references: Requirements.md, Safety.md, Implementation.md
**AI_Philosophy.md** references: AI_Collaborative_Engineering.md
**AI_Collaborative_Engineering.md** references: AI_Philosophy.md, SystematicEngineeringTool.md

See individual documents for complete cross-reference information.
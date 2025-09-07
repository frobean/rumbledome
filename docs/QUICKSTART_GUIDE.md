# Systematic Engineering Platform - Quick Start Guide

**From Zero to Generative Engineering in 30 Minutes**

This guide gets you from "interesting idea" to "working systematic engineering" as fast as possible.

---

## Prerequisites

- Python 3.7+
- Basic understanding of your engineering domain
- Existing project documentation (even rough notes work)

## Step 1: Get the Framework (5 minutes)

### Download the Core Files
```bash
# Download these files from RumbleDome project:
wget https://github.com/your-project/systematic-engineering-core.py
wget https://github.com/your-project/project-config-template.json
```

### Or Copy from RumbleDome
```bash
cp rumbledome/tools/systematic_engineering_core.py ./tools/
cp rumbledome/tools/project-config.json ./tools/my-project-config.json
```

## Step 2: Create Your Project Configuration (10 minutes)

### Minimal Configuration Template
```json
{
  "project_name": "MyProject",
  "project_description": "Brief description of what you're building",
  "traceability_schema": {
    "tiers": ["REQ", "ARCH", "IMPL"],
    "categories": ["CONTROL", "SAFETY", "DATA", "UI"],
    "id_format": "{tier}-{category}-{number:03d}"
  },
  "code_generators": {
    "my-module": {
      "template": "hal_implementation",
      "traceability_ids": ["IMPL-CONTROL-001"],
      "target_struct": "MyModuleImpl",
      "description": "What this module does"
    }
  },
  "validation_rules": {
    "required_derivation_fields": ["Derived From", "Purpose"],
    "cross_reference_targets": ["Requirements.md", "Architecture.md"],
    "safety_critical_patterns": ["safety", "critical", "fault"]
  }
}
```

### Customize for Your Domain
Replace these with your specific needs:
- **Project name**: Your actual project name
- **Categories**: Your engineering domains (MOTOR, SENSOR, COMM, etc.)
- **Safety patterns**: Words that indicate safety-critical areas
- **File targets**: Your actual documentation files

## Step 3: Add Traceability to Documentation (10 minutes)

### Add IDs to Existing Documentation
Take any existing documentation and add traceability IDs:

**Before:**
```markdown
# Motor Control System
The motor controller uses PWM to control speed.
```

**After:**
```markdown
# Motor Control System

**🔗 ARCH-CONTROL-001**: **PWM Motor Speed Control**  
**Derived From**: REQ-CONTROL-001 (Variable Speed Requirements)  
**Purpose**: Uses PWM duty cycle to control motor speed from 0-100%

The motor controller uses PWM to control speed.
```

### ID Naming Convention
- **REQ-**: Requirements (what the system must do)
- **ARCH-**: Architecture (how the system is designed)  
- **IMPL-**: Implementation (specific code/hardware details)

## Step 4: Test Your Setup (5 minutes)

### Create Simple CLI
```python
#!/usr/bin/env python3
import sys
from systematic_engineering_core import SystematicEngineeringFramework

framework = SystematicEngineeringFramework("tools/my-project-config.json")

if sys.argv[1] == "validate":
    framework.validate_all()
elif sys.argv[1] == "generate":
    framework.generate_module(sys.argv[2])
```

### Run Validation
```bash
python3 my-cli.py validate
```

**Expected Output:**
```
🎯 MyProject Engineering Validation
✅ Framework loaded and config validated
✅ All systematic engineering requirements validated
💡 Health Score: 100%
```

## Step 5: Generate Your First Module (Bonus)

### Add Generator Configuration
```json
{
  "code_generators": {
    "controller": {
      "template": "hal_implementation", 
      "target_struct": "MyController",
      "trait_impl": "crate::ControllerTrait",
      "description": "Main control logic"
    }
  }
}
```

### Generate Code
```bash
python3 my-cli.py generate controller
```

**You'll get generated Rust code:**
```rust
//! Generated HAL Implementation
//! 
//! 🔗 Generated from project specifications

pub struct MyController {
    initialized: bool,
}

impl MyController {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}
```

## Common First Steps

### 1. Start with Validation Only
Focus on getting clean validation before code generation:
```bash
# Just validate - no code generation yet
python3 my-cli.py validate
```

### 2. Add IDs Incrementally  
Don't try to add traceability to everything at once:
```markdown
# Add to just one section first
**🔗 ARCH-DATA-001**: **Database Schema Design**
```

### 3. Use Existing Documentation
Work with what you have - don't rewrite everything:
- Requirements documents → REQ- IDs
- Architecture docs → ARCH- IDs  
- Implementation notes → IMPL- IDs

### 4. Build Team Buy-in
Show the value quickly:
```bash
# Show the health score improvement
python3 my-cli.py validate
# 🎯 Health Score: 85% → 95% after adding traceability
```

## Troubleshooting

### "ModuleNotFoundError: systematic_engineering_core"
Make sure the file is in your Python path:
```bash
PYTHONPATH=./tools python3 my-cli.py validate
```

### "Project config not found"
Check the path in your CLI script matches your config file location.

### "No generators available"
Make sure your `code_generators` section has at least one entry.

### Validation Errors
Start simple - the framework is strict but forgiving:
```json
{
  "validation_rules": {
    "required_derivation_fields": ["Purpose"],  // Start with just one field
    "cross_reference_targets": []               // Empty list = no validation
  }
}
```

## Next Steps

### Immediate (Week 1)
- [ ] Get basic validation working
- [ ] Add traceability IDs to 1-2 key documents  
- [ ] Generate your first module
- [ ] Show results to your team

### Short-term (Month 1)
- [ ] Expand to more documentation
- [ ] Create domain-specific templates
- [ ] Add to CI/CD pipeline
- [ ] Train team on traceability practices

### Long-term (Quarter 1)
- [ ] Full project coverage
- [ ] Custom validation rules
- [ ] Integration with requirements tools
- [ ] Measure engineering quality improvements

## Getting Help

### Common Patterns
Look at the RumbleDome configuration for examples:
- Safety-critical systems: RumbleDome boost controller
- Multi-module projects: Hardware abstraction layers
- Complex validation: Cross-reference checking

### Template Ideas
Based on your domain:
- **Embedded systems**: HAL implementations, drivers, protocols
- **Web applications**: API handlers, database models, UI components  
- **Control systems**: Controllers, sensors, actuators
- **Data systems**: Parsers, validators, transforms

### Framework Extension
The framework is designed to be extended:
```python
# Add custom templates
class MyCustomTemplate(CodeTemplate):
    def generate(self, config, specs):
        return "// My custom generated code"

framework.add_custom_template("my_template", MyCustomTemplate())
```

---

**🎯 Goal**: Get from "this looks interesting" to "this is working in our project" in 30 minutes or less.

**💡 Key Insight**: Start small, build incrementally, show value quickly.

The framework is as sophisticated as you need it to be, but it works great even with simple configurations.

---

*Part of the Systematic Engineering Platform - Your systematic engineering journey starts here.*
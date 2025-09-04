# 🚀 Systematic Engineering Quick Start

**TL;DR**: Use `./tools/cli` to make systematic engineering easier than vibe coding.

## ⚡ Essential Commands (Bookmark This!)

```bash
# Check systematic engineering health
./tools/cli audit

# Validate before commit (or pre-commit hook does this automatically)  
./tools/cli validate

# Generate architect report
./tools/cli report --export status.md

# Interactive fix assistant
./tools/cli fix --git    # Creates safe branch for fixes
./tools/cli fix          # Interactive issue resolution
```

## 🎯 Daily Workflow

### When Starting Development
```bash
./tools/cli audit    # Check current health (takes 2 seconds)
```

### When Committing Changes  
```bash
git add .
git commit -m "Your changes"
# → Pre-commit hook automatically validates
# → Commit blocked if systematic engineering issues found
```

### If Commit Blocked
```bash
# Tool provides this exact guidance:
./tools/cli fix --git    # Create safe branch for fixes
./tools/cli fix          # Interactive fix assistant  
./tools/cli validate     # Re-check after fixes
```

## 📊 Understanding Output

### Health Score
- **90-100%**: 🟢 Perfect - keep it up!
- **70-89%**: 🟡 Minor issues - fix when convenient
- **50-69%**: 🟠 Needs attention - schedule fix session
- **<50%**: 🔴 Critical - immediate architect review

### Issue Types
- **Duplicate IDs**: Same traceability ID in multiple places - need consolidation
- **Missing Derivations**: T2+ concepts lack "Derived From" links - need documentation  
- **Broken Cross-refs**: Dead links in documentation - need fixing

## 🔧 Current Status

**RumbleDome Systematic Engineering Status** (as of tool implementation):
- 📄 **16 documents** with systematic traceability
- 🔗 **46 traceability IDs** tracking engineering decisions  
- ⚠️ **60 issues** detected and ready for resolution
- 🛡️ **Pre-commit protection** active and blocking vibe coding

## 🤖 AI Collaboration

### Before Using AI for RumbleDome Work
```bash
./tools/cli audit    # Ensures clean starting point
./tools/cli report --export context-for-ai.md  # Give AI current status
```

### After AI Implementation  
```bash
./tools/cli validate  # Confirms AI maintained discipline
```

## 🎯 Why This Matters

**Without Systematic Engineering:**
```
Developer: "I need boost control"
AI: *generates plausible-looking but potentially dangerous code*
Result: Technical debt, safety issues, unmaintainable code
```

**With Systematic Engineering:**
```
Developer: "I need boost control"
Tool: "Found T2-CONTROL-003 boost control specification"
AI: "Implementing T2-CONTROL-003 exactly as specified..."
Result: Safe, traceable, maintainable implementation
```

## 🚨 Red Flags

**Stop and fix immediately if you see:**
- Health score drops below 50%
- More than 10 new duplicate IDs  
- Pre-commit hook being bypassed with `git commit --no-verify`
- AI generating code without referencing T1→T2→T3→T4 specifications

## 💡 Pro Tips

1. **Run `./tools/cli audit` at start of each work session** - takes 2 seconds
2. **Let pre-commit hook catch issues** - faster than manual checking
3. **Use `./tools/cli fix --git`** - creates safe branch for fixes
4. **Export reports for stakeholders** - demonstrates systematic engineering value  
5. **Health score >90%** - target for optimal development velocity

## 🔗 Full Documentation

- **Complete Guide**: [SystematicEngineeringTool.md](docs/SystematicEngineeringTool.md)
- **AI Collaboration**: [AI_Philosophy.md](docs/AI_Philosophy.md)  
- **Project Overview**: [README.md](README.md)

---

**🎯 Remember**: The tool makes systematic engineering **easier than vibe coding**. Use it!
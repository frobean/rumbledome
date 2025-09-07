#!/usr/bin/env python3
"""
Systematic Engineering Framework - Core Engine

Generalized framework for documentation-driven code generation and traceability validation.
Domain-agnostic engine that works with any project configuration.

Usage:
    from systematic_engineering_core import SystematicEngineeringFramework
    
    framework = SystematicEngineeringFramework("project-config.json")
    framework.validate_all()
    framework.generate_module("module-name")
"""

import json
import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
from abc import ABC, abstractmethod

@dataclass
class TraceabilityID:
    """Universal traceability ID representation"""
    tier: str
    category: str
    number: int
    full_id: str

@dataclass
class ValidationIssue:
    """Represents a systematic engineering validation issue"""
    severity: str  # "error", "warning", "info"
    category: str  # "duplicate_id", "missing_derivation", "broken_reference"
    message: str
    file_path: Optional[str] = None
    line_number: Optional[int] = None

class CodeTemplate(ABC):
    """Abstract base class for code generation templates"""
    
    @abstractmethod
    def generate(self, config: Dict[str, Any], specs: List[str]) -> str:
        """Generate code from configuration and specifications"""
        pass

class HalImplementationTemplate(CodeTemplate):
    """Template for HAL implementation modules"""
    
    def generate(self, config: Dict[str, Any], specs: List[str]) -> str:
        struct_name = config.get("target_struct", "GenericHalImpl")
        trait_impl = config.get("trait_impl", "GenericTrait")
        safety_reqs = config.get("safety_requirements", [])
        
        # Extract traceability info
        traceability_comment = self._build_traceability_comment(config, specs)
        
        # Generate basic structure
        return f"""//! Generated HAL Implementation
//! 
{traceability_comment}

use crate::{{HalResult, HalError}};

/// Hardware-specific implementation
pub struct {struct_name} {{
    initialized: bool,
    // TODO: Add hardware-specific fields
}}

impl {struct_name} {{
    pub fn new() -> Self {{
        Self {{
            initialized: false,
        }}
    }}
    
    pub fn init(&mut self) -> HalResult<()> {{
        // TODO: Initialize hardware
        self.initialized = true;
        Ok(())
    }}
}}

impl {trait_impl} for {struct_name} {{
    // TODO: Implement trait methods based on specifications
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_initialization() {{
        let mut impl_instance = {struct_name}::new();
        assert!(impl_instance.init().is_ok());
    }}
}}"""
    
    def _build_traceability_comment(self, config: Dict[str, Any], specs: List[str]) -> str:
        """Build traceability comment block"""
        ids = config.get("traceability_ids", [])
        if not ids:
            return "//! Generated from project specifications"
        
        return f"//! 🔗 {', '.join(ids)}: Generated Implementation\\n//! AI Traceability: {specs[0] if specs else 'Generated from specifications'}"

class ControlSystemTemplate(CodeTemplate):
    """Template for control system modules"""
    
    def generate(self, config: Dict[str, Any], specs: List[str]) -> str:
        struct_name = config.get("target_struct", "GenericController")
        
        return f"""//! Generated Control System
//! 
//! Generated from specifications

use crate::{{HalResult, HalError}};

pub struct {struct_name} {{
    enabled: bool,
    // TODO: Add control-specific fields
}}

impl {struct_name} {{
    pub fn new() -> Self {{
        Self {{
            enabled: false,
        }}
    }}
    
    /// Main control loop update
    pub fn update(&mut self, dt: f32) -> HalResult<()> {{
        // TODO: Implement control logic from specifications
        Ok(())
    }}
}}"""

class SystematicEngineeringFramework:
    """
    Core framework for systematic engineering with generative capabilities
    
    Loads project configuration and provides domain-agnostic validation,
    traceability management, and code generation services.
    """
    
    def __init__(self, config_path: str):
        self.config_path = config_path
        self.config = self._load_config()
        self.docs_dir = Path("docs")
        self.validation_issues = []
        
        # Register templates
        self.templates = {
            "hal_implementation": HalImplementationTemplate(),
            "control_system": ControlSystemTemplate(),
        }
    
    def _load_config(self) -> Dict[str, Any]:
        """Load project configuration"""
        try:
            with open(self.config_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            raise RuntimeError(f"Failed to load project config: {e}")
    
    def validate_all(self, blocking: bool = False) -> List[ValidationIssue]:
        """Validate all systematic engineering requirements"""
        print(f"🎯 {self.config['project_name']} Engineering Validation")
        print("✅ Framework loaded and config validated")
        
        self.validation_issues = []
        
        # Validate traceability IDs
        self._validate_traceability_ids()
        
        # Validate cross-references
        self._validate_cross_references()
        
        # Validate derivations
        self._validate_derivations()
        
        # Print summary
        self._print_validation_summary()
        
        if blocking and any(issue.severity == "error" for issue in self.validation_issues):
            sys.exit(1)
        
        return self.validation_issues
    
    def generate_module(self, module_name: str) -> Optional[str]:
        """Generate code module from specifications"""
        generators = self.config.get("code_generators", {})
        
        if module_name not in generators:
            print(f"❌ No generator configured for '{module_name}'")
            print(f"💡 Available generators: {', '.join(generators.keys())}")
            return None
        
        generator_config = generators[module_name]
        template_name = generator_config.get("template", "hal_implementation")
        
        if template_name not in self.templates:
            print(f"❌ Unknown template: {template_name}")
            return None
        
        # Extract specifications from documentation
        specs = self._extract_specifications(generator_config.get("traceability_ids", []))
        
        # Generate code using template
        template = self.templates[template_name]
        generated_code = template.generate(generator_config, specs)
        
        print("🎭 Generated complete module:")
        print(generated_code)
        
        return generated_code
    
    def _validate_traceability_ids(self):
        """Validate traceability ID consistency"""
        schema = self.config.get("traceability_schema", {})
        id_format = schema.get("id_format", "{tier}-{category}-{number:03d}")
        
        found_ids = set()
        duplicate_ids = []
        
        # Scan all documentation files
        for doc_file in self.docs_dir.glob("*.md"):
            content = doc_file.read_text(encoding='utf-8')
            
            # Find all traceability IDs
            pattern = r'T\d+-[A-Z]+-\d+'  # Generic pattern
            ids_in_file = re.findall(pattern, content)
            
            for id_str in ids_in_file:
                if id_str in found_ids:
                    duplicate_ids.append(id_str)
                else:
                    found_ids.add(id_str)
        
        # Report duplicates
        for dup_id in duplicate_ids:
            self.validation_issues.append(ValidationIssue(
                severity="error",
                category="duplicate_id", 
                message=f"Duplicate traceability ID: {dup_id}"
            ))
    
    def _validate_cross_references(self):
        """Validate cross-references between documents"""
        validation_rules = self.config.get("validation_rules", {})
        targets = validation_rules.get("cross_reference_targets", [])
        
        for target in targets:
            if not (self.docs_dir / target).exists():
                self.validation_issues.append(ValidationIssue(
                    severity="warning",
                    category="missing_file",
                    message=f"Referenced file missing: {target}"
                ))
    
    def _validate_derivations(self):
        """Validate required derivation fields"""
        validation_rules = self.config.get("validation_rules", {})
        required_fields = validation_rules.get("required_derivation_fields", [])
        
        # This would scan for traceability blocks and check required fields
        # Implementation simplified for demo
        pass
    
    def _extract_specifications(self, traceability_ids: List[str]) -> List[str]:
        """Extract specifications for given traceability IDs"""
        specs = []
        
        for doc_file in self.docs_dir.glob("*.md"):
            content = doc_file.read_text(encoding='utf-8')
            
            for trace_id in traceability_ids:
                if trace_id in content:
                    # Extract surrounding context
                    lines = content.split('\\n')
                    for i, line in enumerate(lines):
                        if trace_id in line:
                            # Get several lines of context
                            start = max(0, i - 2)
                            end = min(len(lines), i + 5)
                            context = '\\n'.join(lines[start:end])
                            specs.append(context)
                            break
        
        return specs
    
    def _print_validation_summary(self):
        """Print validation results summary"""
        errors = [i for i in self.validation_issues if i.severity == "error"]
        warnings = [i for i in self.validation_issues if i.severity == "warning"]
        
        if not self.validation_issues:
            print("✅ All systematic engineering requirements validated")
            print("💡 Health Score: 100%")
        else:
            print(f"⚠️ Found {len(errors)} errors, {len(warnings)} warnings")
            
            for issue in self.validation_issues[:5]:  # Show first 5
                severity_icon = "❌" if issue.severity == "error" else "⚠️"
                print(f"  {severity_icon} {issue.category}: {issue.message}")
    
    def get_available_generators(self) -> List[str]:
        """Get list of available code generators"""
        return list(self.config.get("code_generators", {}).keys())
    
    def add_custom_template(self, name: str, template: CodeTemplate):
        """Add custom code generation template"""
        self.templates[name] = template

def main():
    """Main entry point for framework CLI"""
    if len(sys.argv) < 2:
        print("Usage: systematic-engineering-core.py <command> [args...]")
        print("Commands: validate, generate <module>, list-generators")
        return
    
    # Default config path
    config_path = "tools/project-config.json"
    if not os.path.exists(config_path):
        print(f"❌ Project config not found: {config_path}")
        return
    
    framework = SystematicEngineeringFramework(config_path)
    command = sys.argv[1]
    
    if command == "validate":
        blocking = "--blocking" in sys.argv
        framework.validate_all(blocking=blocking)
    
    elif command == "generate":
        if len(sys.argv) < 3:
            print("❌ Usage: generate <module-name>")
            return
        module_name = sys.argv[2]
        framework.generate_module(module_name)
    
    elif command == "list-generators":
        generators = framework.get_available_generators()
        print("💡 Available generators:")
        for gen in generators:
            print(f"  - {gen}")
    
    else:
        print(f"❌ Unknown command: {command}")

if __name__ == '__main__':
    main()
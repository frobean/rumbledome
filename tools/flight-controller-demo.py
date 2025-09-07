#!/usr/bin/env python3
"""
Flight Controller Demo - Systematic Engineering Framework

Demonstrates portability of the framework with a completely different engineering domain.
Shows how the same framework can generate flight control systems instead of boost controllers.
"""

import sys
import os
from pathlib import Path

# Add current directory for imports
sys.path.insert(0, str(Path(__file__).parent))

from systematic_engineering_core import SystematicEngineeringFramework

def main():
    """Demonstrate framework with flight controller configuration"""
    
    print("🚁 Flight Controller Engineering Framework Demo")
    print("=" * 60)
    
    # Use flight controller config
    config_path = "tools/example-flight-controller-config.json"
    
    try:
        framework = SystematicEngineeringFramework(config_path)
        print("✅ Flight controller framework loaded successfully")
        
        # Show available generators
        print("\\n💡 Available Flight Control Generators:")
        generators = framework.get_available_generators()
        
        config = framework.config
        for gen in generators:
            gen_config = config["code_generators"][gen]
            desc = gen_config.get("description", "No description")
            safety = "🔒 SAFETY-CRITICAL" if gen_config.get("safety_critical") else "⚡ STANDARD"
            print(f"  - {gen}: {desc} [{safety}]")
        
        print("\\n🔧 Generating attitude controller module...")
        print("-" * 50)
        
        # Generate a flight control module
        result = framework.generate_module("attitude-controller")
        
        if result:
            print("\\n✅ Successfully generated flight control module!")
            print("\\n📊 Domain Comparison:")
            print("  RumbleDome    → Boost pressure control")
            print("  Flight Control → Aircraft attitude control")
            print("  Same Framework → Different domains, same methodology")
        
        print("\\n🎯 Framework Validation:")
        issues = framework.validate_all()
        
        if not issues:
            print("✅ Flight controller project would have perfect health!")
        else:
            print(f"⚠️ Found validation issues (expected for demo)")
    
    except Exception as e:
        print(f"❌ Demo failed: {e}")
        return 1
    
    print("\\n" + "=" * 60)
    print("🌟 Framework Portability Demonstrated!")
    print("   Same core engine, different domains")
    print("   Configuration-driven, not code-driven")
    print("   Reusable across ANY engineering project")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
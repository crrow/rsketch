#!/usr/bin/env python3
"""
Advanced Dependency Analyzer
Provides detailed analysis of dependency updates similar to Dependabot
"""

import json
import subprocess
import sys
import os
import argparse
from typing import Dict, List, Optional, Tuple
from pathlib import Path
import re

class DependencyAnalyzer:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.results = {}
    
    def run_command(self, cmd: List[str], cwd: Optional[Path] = None) -> Tuple[int, str, str]:
        """Run a command and return exit code, stdout, stderr"""
        try:
            result = subprocess.run(
                cmd, 
                cwd=cwd or self.project_root,
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return 1, "", "Command timed out"
        except Exception as e:
            return 1, "", str(e)
    
    def analyze_cargo_deps(self) -> Dict:
        """Analyze Rust dependencies using cargo-outdated"""
        print("🦀 Analyzing Rust dependencies...")
        
        # Check if cargo-outdated is installed
        ret_code, _, _ = self.run_command(["cargo", "outdated", "--version"])
        if ret_code != 0:
            print("⚠️  cargo-outdated not found. Install with: cargo install cargo-outdated")
            return {"error": "cargo-outdated not installed"}
        
        # Get outdated dependencies in JSON format
        ret_code, stdout, stderr = self.run_command([
            "cargo", "outdated", "--workspace", "--format", "json"
        ])
        
        if ret_code != 0:
            return {"error": f"Failed to run cargo outdated: {stderr}"}
        
        try:
            data = json.loads(stdout)
            dependencies = data.get("dependencies", [])
            
            result = {
                "total_deps": len(dependencies),
                "outdated_count": 0,
                "updates": [],
                "security_advisories": []
            }
            
            for dep in dependencies:
                if dep["project"] != dep["latest"]:
                    result["outdated_count"] += 1
                    
                    # Calculate semver change type
                    change_type = self.determine_semver_change(dep["project"], dep["latest"])
                    
                    update_info = {
                        "name": dep["name"],
                        "current": dep["project"],
                        "latest": dep["latest"],
                        "change_type": change_type,
                        "kind": dep.get("kind", "unknown")
                    }
                    result["updates"].append(update_info)
            
            return result
            
        except json.JSONDecodeError:
            return {"error": "Failed to parse cargo outdated output"}
    
    def analyze_go_deps(self, module_path: Path) -> Dict:
        """Analyze Go dependencies in a specific module"""
        if not (module_path / "go.mod").exists():
            return {"error": "go.mod not found"}
        
        print(f"🐹 Analyzing Go dependencies in {module_path.name}...")
        
        # Get list of modules with updates available
        ret_code, stdout, stderr = self.run_command([
            "go", "list", "-u", "-m", "all"
        ], cwd=module_path)
        
        if ret_code != 0:
            return {"error": f"Failed to run go list: {stderr}"}
        
        result = {
            "module_path": str(module_path),
            "total_deps": 0,
            "outdated_count": 0,
            "updates": []
        }
        
        for line in stdout.strip().split('\n'):
            if not line:
                continue
                
            result["total_deps"] += 1
            
            # Parse go list output for updates
            # Format: module_name version [latest_version]
            match = re.match(r'^(\S+)\s+(\S+)(?:\s+\[([^\]]+)\])?', line)
            if match:
                module_name, current_version, latest_version = match.groups()
                
                if latest_version and latest_version != current_version:
                    result["outdated_count"] += 1
                    
                    change_type = self.determine_semver_change(current_version, latest_version)
                    
                    update_info = {
                        "name": module_name,
                        "current": current_version,
                        "latest": latest_version,
                        "change_type": change_type
                    }
                    result["updates"].append(update_info)
        
        return result
    
    def determine_semver_change(self, current: str, latest: str) -> str:
        """Determine the type of semantic version change"""
        try:
            # Simple semver parsing (handles most cases)
            current_parts = self.parse_version(current)
            latest_parts = self.parse_version(latest)
            
            if current_parts[0] != latest_parts[0]:
                return "major"
            elif current_parts[1] != latest_parts[1]:
                return "minor"
            elif current_parts[2] != latest_parts[2]:
                return "patch"
            else:
                return "other"
        except:
            return "unknown"
    
    def parse_version(self, version: str) -> Tuple[int, int, int]:
        """Parse a version string into major.minor.patch"""
        # Remove common prefixes
        version = version.lstrip('v')
        
        # Split on dots and take first 3 parts
        parts = version.split('.')[:3]
        
        # Convert to integers, default to 0 if not parseable
        result = []
        for part in parts:
            try:
                # Remove any non-numeric suffixes (like -alpha, +build)
                numeric_part = re.match(r'(\d+)', part)
                if numeric_part:
                    result.append(int(numeric_part.group(1)))
                else:
                    result.append(0)
            except:
                result.append(0)
        
        # Ensure we have 3 parts
        while len(result) < 3:
            result.append(0)
        
        return tuple(result[:3])
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive dependency report"""
        report = {
            "timestamp": subprocess.check_output(["date", "+%Y-%m-%d %H:%M:%S"]).decode().strip(),
            "project_root": str(self.project_root),
            "rust": {},
            "go_modules": {},
            "summary": {}
        }
        
        # Analyze Rust dependencies
        if (self.project_root / "Cargo.toml").exists():
            report["rust"] = self.analyze_cargo_deps()
        
        # Analyze Go modules
        go_module_dirs = [
            self.project_root / "bindings" / "go",
            self.project_root / "examples" / "goclient"
        ]
        
        for module_dir in go_module_dirs:
            if module_dir.exists():
                module_name = f"{module_dir.parent.name}/{module_dir.name}"
                report["go_modules"][module_name] = self.analyze_go_deps(module_dir)
        
        # Generate summary
        total_rust_updates = report["rust"].get("outdated_count", 0)
        total_go_updates = sum(
            module.get("outdated_count", 0) 
            for module in report["go_modules"].values()
        )
        
        report["summary"] = {
            "total_rust_updates": total_rust_updates,
            "total_go_updates": total_go_updates,
            "total_updates": total_rust_updates + total_go_updates,
            "has_major_updates": self.has_major_updates(report),
            "has_security_updates": self.has_security_updates(report)
        }
        
        return report
    
    def has_major_updates(self, report: Dict) -> bool:
        """Check if there are any major version updates"""
        # Check Rust dependencies
        rust_updates = report.get("rust", {}).get("updates", [])
        if any(update.get("change_type") == "major" for update in rust_updates):
            return True
        
        # Check Go dependencies
        for module in report.get("go_modules", {}).values():
            go_updates = module.get("updates", [])
            if any(update.get("change_type") == "major" for update in go_updates):
                return True
        
        return False
    
    def has_security_updates(self, report: Dict) -> bool:
        """Check if there are any security-related updates"""
        # This would require integration with security databases
        # For now, return False but structure is in place
        return False
    
    def print_report(self, report: Dict):
        """Print a human-readable report"""
        print("\n" + "="*60)
        print("📊 DEPENDENCY ANALYSIS REPORT")
        print("="*60)
        
        summary = report["summary"]
        print(f"📅 Generated: {report['timestamp']}")
        print(f"📁 Project: {Path(report['project_root']).name}")
        print(f"🔄 Total Updates Available: {summary['total_updates']}")
        
        if summary['has_major_updates']:
            print("⚠️  Major version updates detected!")
        
        print()
        
        # Rust section
        if "rust" in report and "updates" in report["rust"]:
            rust_data = report["rust"]
            print(f"🦀 Rust Dependencies ({rust_data.get('outdated_count', 0)} updates)")
            print("-" * 40)
            
            if rust_data.get("error"):
                print(f"❌ Error: {rust_data['error']}")
            elif rust_data.get("updates"):
                for update in rust_data["updates"]:
                    change_icon = self.get_change_icon(update["change_type"])
                    print(f"  {change_icon} {update['name']}: {update['current']} → {update['latest']}")
            else:
                print("  ✅ All dependencies up to date")
            print()
        
        # Go modules section
        if "go_modules" in report:
            for module_name, module_data in report["go_modules"].items():
                print(f"🐹 Go Module: {module_name} ({module_data.get('outdated_count', 0)} updates)")
                print("-" * 40)
                
                if module_data.get("error"):
                    print(f"❌ Error: {module_data['error']}")
                elif module_data.get("updates"):
                    for update in module_data["updates"]:
                        change_icon = self.get_change_icon(update["change_type"])
                        print(f"  {change_icon} {update['name']}: {update['current']} → {update['latest']}")
                else:
                    print("  ✅ All dependencies up to date")
                print()
        
        print("💡 Run 'just deps-update' to update dependencies")
        print("💡 Run 'just deps-check' for a dry-run")
    
    def get_change_icon(self, change_type: str) -> str:
        """Get an icon for the type of version change"""
        icons = {
            "major": "🔴",
            "minor": "🟡", 
            "patch": "🟢",
            "other": "🔵",
            "unknown": "⚪"
        }
        return icons.get(change_type, "⚪")

def main():
    parser = argparse.ArgumentParser(description="Analyze project dependencies")
    parser.add_argument("--output", "-o", choices=["json", "human"], default="human",
                       help="Output format (default: human)")
    parser.add_argument("--save", "-s", metavar="FILE",
                       help="Save JSON report to file")
    parser.add_argument("--project-root", default=".",
                       help="Project root directory (default: current directory)")
    
    args = parser.parse_args()
    
    project_root = Path(args.project_root).resolve()
    
    if not project_root.exists():
        print(f"❌ Project root not found: {project_root}")
        sys.exit(1)
    
    analyzer = DependencyAnalyzer(project_root)
    report = analyzer.generate_report()
    
    if args.output == "json":
        print(json.dumps(report, indent=2))
    else:
        analyzer.print_report(report)
    
    if args.save:
        with open(args.save, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\n💾 Report saved to: {args.save}")
    
    # Exit with non-zero if updates are available (useful for CI)
    if report["summary"]["total_updates"] > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Test script to verify all Nest MCP servers are working correctly.

Run from repository root:
    .venv/bin/python tools/test_mcp_servers.py
"""

import sys
import os
from pathlib import Path

# Add tools directory to path
sys.path.insert(0, str(Path(__file__).parent))

def test_memory_server():
    """Test nest-memory server (project docs search)."""
    print("\n🔍 Testing nest-memory server...")
    try:
        import subprocess
        result = subprocess.run(
            [".venv/bin/python", "tools/search_memory.py", "nest-core architecture"],
            capture_output=True,
            text=True,
            timeout=10,
            cwd=Path(__file__).parent.parent
        )
        if result.returncode == 0 and result.stdout.strip():
            lines = [l for l in result.stdout.split('\n') if l.strip() and not l.startswith('---')]
            print(f"   ✓ Search returned results ({len(lines)} lines)")
            return True
        else:
            print(f"   ✗ Search failed: {result.stderr[:200]}")
            return False
    except Exception as e:
        print(f"   ✗ Error: {e}")
        return False

def test_knowledge_server():
    """Test nest-knowledge server (Rust/Tauri/React/Tailwind docs)."""
    print("\n📚 Testing nest-knowledge server...")
    try:
        import subprocess
        result = subprocess.run(
            [".venv/bin/python", "tools/search_knowledge.py", "async trait", "--collection", "rust-book"],
            capture_output=True,
            text=True,
            timeout=10,
            cwd=Path(__file__).parent.parent
        )
        if result.returncode == 0 and result.stdout.strip():
            lines = [l for l in result.stdout.split('\n') if l.strip() and not l.startswith('---')]
            print(f"   ✓ Search returned results ({len(lines)} lines)")
            return True
        else:
            print(f"   ✗ Search failed: {result.stderr[:200]}")
            return False
    except Exception as e:
        print(f"   ✗ Error: {e}")
        return False

def test_context_memory_server():
    """Test nest-context-memory server (session context)."""
    print("\n💾 Testing nest-context-memory server...")
    try:
        from context_memory import save_context, search_context, list_context
        
        # Test save
        test_title = "MCP Test Verification"
        result = save_context(
            content="Test entry to verify MCP server is working",
            title=test_title,
            session_key="mcp-test-session",
            tags=["test", "verification", "mcp"]
        )
        print(f"   ✓ Saved test entry (ID: {result})")
        
        # Test search
        results = search_context(query="test entry", session_key="mcp-test-session", limit=3)
        if results:
            print(f"   ✓ Search found {len(results)} results")
        else:
            print(f"   ✗ Search returned no results")
            return False
        
        # Test list
        recent = list_context(session_key="mcp-test-session", limit=5)
        if recent:
            print(f"   ✓ List returned {len(recent)} recent entries")
        else:
            print(f"   ✗ List returned no results")
            return False
        
        return True
    except Exception as e:
        print(f"   ✗ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_mcp_protocol():
    """Test MCP protocol handshake for all servers."""
    print("\n🔌 Testing MCP protocol handshake...")
    
    import subprocess
    import json
    
    servers = {
        "nest-memory": "tools/mcp_memory_server.py",
        "nest-context-memory": "tools/mcp_context_memory_server.py",
        "nest-knowledge": "tools/mcp_knowledge_server.py",
    }
    
    initialize_msg = json.dumps({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        },
        "id": 1
    })
    
    all_ok = True
    for name, script in servers.items():
        try:
            result = subprocess.run(
                [".venv/bin/python", script],
                input=initialize_msg,
                capture_output=True,
                text=True,
                timeout=3,
                cwd=Path(__file__).parent.parent
            )
            
            response = json.loads(result.stdout.strip())
            if response.get("result", {}).get("serverInfo", {}).get("name") == name:
                print(f"   ✓ {name} responded correctly")
            else:
                print(f"   ✗ {name} unexpected response")
                all_ok = False
        except Exception as e:
            print(f"   ✗ {name} error: {e}")
            all_ok = False
    
    return all_ok

def main():
    """Run all MCP server tests."""
    print("=" * 60)
    print("Nest MCP Server Verification")
    print("=" * 60)
    
    # Check prerequisites
    print("\n📋 Checking prerequisites...")
    env_file = Path(__file__).parent.parent / ".env"
    if not env_file.exists():
        print("   ✗ .env file not found (copy .env.example to .env)")
        return False
    print("   ✓ .env file exists")
    
    venv_python = Path(__file__).parent.parent / ".venv" / "bin" / "python"
    if not venv_python.exists():
        print("   ✗ Python venv not found (run setup from MCP-SETUP.md)")
        return False
    print("   ✓ Python venv exists")
    
    # Run tests
    results = []
    
    results.append(("MCP Protocol", test_mcp_protocol()))
    results.append(("Project Memory", test_memory_server()))
    results.append(("Knowledge Base", test_knowledge_server()))
    results.append(("Context Memory", test_context_memory_server()))
    
    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)
    
    passed = sum(1 for _, ok in results if ok)
    total = len(results)
    
    for name, ok in results:
        status = "✓ PASS" if ok else "✗ FAIL"
        print(f"  {status}: {name}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n✅ All MCP servers are working correctly!")
        print("\nNext steps:")
        print("  1. Verify .cursor/mcp.json has correct paths")
        print("  2. Reload Cursor window (Ctrl+Shift+P → Developer: Reload Window)")
        print("  3. Check Settings → Tools & MCP for all servers")
        return True
    else:
        print("\n❌ Some tests failed. Check MCP-SETUP.md for troubleshooting.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)

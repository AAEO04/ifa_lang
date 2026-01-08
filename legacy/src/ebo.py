# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ẸBỌ - SACRIFICE & RESOURCE MANAGEMENT                     ║
║                    "The Sacred Offering"                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Ẹbọ (Sacrifice) - Resource lifecycle and garbage collection model.         ║
║  In computing: Resources must be "sacrificed" (freed) to maintain harmony.  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import Any, Callable, Dict, List, Optional
from dataclasses import dataclass, field
from contextlib import contextmanager
import weakref
import gc


@dataclass
class SacrificialResource:
    """A resource that will be sacrificed (freed) when block exits."""
    name: str
    resource: Any
    cleanup_fn: Callable = None
    sacrificed: bool = False
    
    def sacrifice(self):
        """Perform the sacrifice (cleanup)."""
        if self.sacrificed:
            return
        if self.cleanup_fn:
            try:
                self.cleanup_fn(self.resource)
            except Exception as e:
                print(f"  ⚠️ [Ẹbọ] Sacrifice failed for '{self.name}': {e}")
        self.sacrificed = True
        print(f"  🔥 [Ẹbọ] Sacrificed: {self.name}")


class EboBlock:
    """
    Ẹbọ Block - Atomic resource sacrifice block.
    Ensures resources are always cleaned up, similar to try/finally.
    Thread-safe with mutex protection.
    
    Usage:
        with EboBlock("file_op") as ebo:
            file = ebo.acquire("file", open("x.txt"), lambda f: f.close())
            data = file.read()
        # file.close() automatically called
    """
    
    def __init__(self, name: str = "anonymous"):
        import threading
        self.name = name
        self.resources: List[SacrificialResource] = []
        self.active = False
        self._lock = threading.Lock()  # Thread safety
    
    def acquire(self, name: str, resource: Any, cleanup_fn: Callable = None) -> Any:
        """Acquire a resource that will be sacrificed on block exit."""
        with self._lock:
            sr = SacrificialResource(name=name, resource=resource, cleanup_fn=cleanup_fn)
            self.resources.append(sr)
            print(f"  📦 [Ẹbọ] Acquired: {name}")
            return resource
    
    def __enter__(self):
        self.active = True
        print(f"\n╔═══ Ẹbọ Block: {self.name} ═══╗")
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        print(f"╠═══ Sacrifice Phase ═══╣")
        with self._lock:
            for sr in reversed(self.resources):
                sr.sacrifice()
            self.resources.clear()
        self.active = False
        print(f"╚═══ Ẹbọ Complete ═══╝\n")
        return False


class SacrificeRegistry:
    """Global registry for tracking resources awaiting sacrifice."""
    
    def __init__(self):
        self._pending: Dict[int, weakref.ref] = {}
        self._cleanup_fns: Dict[int, Callable] = {}
        self._names: Dict[int, str] = {}
        self._sacrifice_count: int = 0
    
    def register(self, name: str, resource: Any, cleanup_fn: Callable = None):
        """Register a resource for eventual sacrifice."""
        obj_id = id(resource)
        self._pending[obj_id] = weakref.ref(resource)
        self._cleanup_fns[obj_id] = cleanup_fn
        self._names[obj_id] = name
    
    def sacrifice_all(self):
        """Force sacrifice of all pending resources."""
        print("\n╔════════════════════════════════════════════════╗")
        print("║         MASS SACRIFICE (Ẹbọ Ńlá)               ║")
        print("╚════════════════════════════════════════════════╝")
        
        for obj_id in list(self._pending.keys()):
            ref = self._pending.get(obj_id)
            if ref:
                obj = ref()
                if obj is not None:
                    cleanup = self._cleanup_fns.get(obj_id)
                    name = self._names.get(obj_id, "unknown")
                    if cleanup:
                        try:
                            cleanup(obj)
                            print(f"  🔥 Sacrificed: {name}")
                        except Exception as e:
                            print(f"  ⚠️ Failed: {name} - {e}")
                    self._sacrifice_count += 1
            if obj_id in self._pending: del self._pending[obj_id]
            if obj_id in self._cleanup_fns: del self._cleanup_fns[obj_id]
            if obj_id in self._names: del self._names[obj_id]
        gc.collect()
        print(f"\n  Total sacrifices: {self._sacrifice_count}")
    
    def pending_count(self) -> int:
        return len(self._pending)


@contextmanager
def ase_block(name: str = "àṣẹ"):
    """Àṣẹ Block - Atomic execution with guaranteed cleanup."""
    resources = []
    def track(resource_name: str, resource: Any, cleanup: Callable = None):
        resources.append((resource_name, resource, cleanup))
        return resource
    
    print(f"\n⚡ [Àṣẹ] Block started: {name}")
    try:
        yield track
        print(f"✓ [Àṣẹ] Block succeeded: {name}")
    except Exception as e:
        print(f"✗ [Àṣẹ] Block failed: {name} - {e}")
        raise
    finally:
        for res_name, res, cleanup in reversed(resources):
            if cleanup:
                try:
                    cleanup(res)
                    print(f"  🔥 [Àṣẹ] Released: {res_name}")
                except: pass


# Global instances
sacrifice_registry = SacrificeRegistry()

def ebo(name: str = "ẹbọ") -> EboBlock:
    return EboBlock(name)

def sacrifice(resource: Any, cleanup_fn: Callable = None, name: str = "resource"):
    sacrifice_registry.register(name, resource, cleanup_fn)

def sacrifice_all():
    sacrifice_registry.sacrifice_all()


__all__ = ['SacrificialResource', 'EboBlock', 'SacrificeRegistry', 
           'sacrifice_registry', 'ase_block', 'ebo', 'sacrifice', 'sacrifice_all']

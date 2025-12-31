# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                       Ọ̀SÁ - THE RUNNER (CONCURRENCY)                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Domain: Async, Process, Thread Management & Data Serialization              ║
║  The Wind Spirit - Swift movement and parallel execution                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import time
import threading
import json
import csv
from typing import Any, Callable, List, Dict

from .base import OduModule


class OsaDomain(OduModule):
    """
    Ọ̀sá - The Runner / The Wind
    Handles concurrency, async operations, time, and data serialization (JSON/CSV).
    """
    
    # Shared state for concurrency
    _tasks: Dict[int, threading.Thread] = {}
    _task_counter: int = 0
    _locks: Dict[str, threading.Lock] = {}
    
    def __init__(self):
        super().__init__("Ọ̀sá", "0111", "The Runner - Concurrency & Async")
        
        # Concurrency
        self._register("sa", self.sa, "Spawn task/thread")
        self._register("duro", self.duro, "Wait/sleep milliseconds")
        self._register("ago", self.ago, "Get current timestamp (ms)")
        self._register("pa", self.pa, "Kill/stop task")
        
        # Synchronization
        self._register("khoa", self.khoa, "Lock (acquire mutex)")
        self._register("si", self.si, "Unlock (release mutex)")
        
        # Data Serialization - JSON
        self._register("si_json", self.si_json, "Serialize to JSON string")
        self._register("lati_json", self.lati_json, "Parse JSON string")
        
        # Data Serialization - CSV
        self._register("si_csv", self.si_csv, "Write to CSV file")
        self._register("lati_csv", self.lati_csv, "Read from CSV file")
        
        # Spec Function
        self._register("fo", self.fo, "Jump/Goto (control flow placeholder)")
    
    # =========================================================================
    # CONCURRENCY
    # =========================================================================
    
    def sa(self, task_name: str, func: Callable = None, *args) -> int:
        """
        sá (Run/Spawn) - Spawn a new task/thread
        """
        task_id = OsaDomain._task_counter
        OsaDomain._task_counter += 1
        
        if func:
            thread = threading.Thread(target=func, args=args, name=task_name)
            thread.daemon = True  # Allow exit even if running
            thread.start()
            OsaDomain._tasks[task_id] = thread
        
        print(f"[Ọ̀sá] Spawned task: {task_name} (id={task_id})")
        return task_id
    
    def duro(self, ms: int) -> None:
        """Wait for specified milliseconds."""
        time.sleep(ms / 1000)
    
    def ago(self) -> float:
        """Get current timestamp in milliseconds."""
        return time.time() * 1000
    
    def pa(self, task_id: int) -> None:
        """Stop/Kill task (removes reference, can't truly stop Python threads)."""
        if task_id in OsaDomain._tasks:
            del OsaDomain._tasks[task_id]
    
    # =========================================================================
    # SYNCHRONIZATION
    # =========================================================================
    
    def khoa(self, name: str = "global") -> None:
        """Lock (Khoá) - Acquire mutex."""
        if name not in OsaDomain._locks:
            OsaDomain._locks[name] = threading.Lock()
        OsaDomain._locks[name].acquire()
    
    def si(self, name: str = "global") -> None:
        """Unlock (Ṣí) - Release mutex."""
        if name in OsaDomain._locks:
            OsaDomain._locks[name].release()
    
    # =========================================================================
    # DATA SERIALIZATION (The Wind carries info)
    # =========================================================================
    
    def si_json(self, data: Any, indent: int = 2) -> str:
        """Serialize to JSON string."""
        return json.dumps(data, indent=indent, default=str)
    
    def lati_json(self, text: str) -> Any:
        """Parse JSON string."""
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return None
    
    def si_csv(self, filename: str, rows: List[List[Any]]) -> bool:
        """Write to CSV file."""
        try:
            with open(filename, 'w', newline='', encoding='utf-8') as f:
                writer = csv.writer(f)
                writer.writerows(rows)
            return True
        except (IOError, csv.Error):
            return False
    
    def lati_csv(self, filename: str) -> List[List[str]]:
        """Read from CSV file."""
        try:
            with open(filename, 'r', encoding='utf-8') as f:
                reader = csv.reader(f)
                return list(reader)
        except (IOError, csv.Error):
            return []
    
    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def fo(self, label: str = None) -> None:
        """fò() - Jump (Goto). Control flow placeholder."""
        # Python has no goto, this is a placeholder for transpiler support
        pass


# Module-level singleton and functions for backwards compatibility
_osa = OsaDomain()

def sa(task_name: str, func: Callable = None, *args) -> int:
    return _osa.sa(task_name, func, *args)

def duro(ms: int) -> None:
    return _osa.duro(ms)

def ago() -> float:
    return _osa.ago()

def pa(task_id: int) -> None:
    return _osa.pa(task_id)

def khoa(name: str = "global") -> None:
    return _osa.khoa(name)

def si(name: str = "global") -> None:
    return _osa.si(name)

def si_json(data: Any, indent: int = 2) -> str:
    return _osa.si_json(data, indent)

def lati_json(text: str) -> Any:
    return _osa.lati_json(text)

def si_csv(filename: str, rows: List[List[Any]]) -> bool:
    return _osa.si_csv(filename, rows)

def lati_csv(filename: str) -> List[List[str]]:
    return _osa.lati_csv(filename)

def fo(label: str = None) -> None:
    return _osa.fo(label)

# English aliases
spawn = sa
wait = duro
sleep = duro
now = ago
lock = khoa
unlock = si
to_json = si_json
from_json = lati_json

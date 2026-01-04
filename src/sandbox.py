# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ÌGBÁLẸ̀ - THE SACRED GROUND                                ║
║                    Ifá Sandbox Container System                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  A Docker-like sandbox system designed with the 16 Odù domains.              ║
║                                                                              ║
║  Architecture:                                                               ║
║    Ogbè    - Container Initialization & Lifecycle                            ║
║    Ọ̀yẹ̀kú   - Container Termination & Cleanup                                ║
║    Ìwòrì   - Time Limits & Resource Monitoring                               ║
║    Òdí     - Filesystem Isolation & Virtual FS                               ║
║    Ìrosù   - I/O Capture & Logging                                           ║
║    Ọ̀wọ́nrín - Entropy & Randomness Isolation                                 ║
║    Ọ̀bàrà   - CPU Resource Limits                                             ║
║    Ọ̀kànràn - Error Handling & Security Violations                           ║
║    Ògúndá  - Process Control & Fork Limits                                   ║
║    Ọ̀sá     - Network Namespace Isolation                                     ║
║    Ìká     - String/Data Sanitization                                        ║
║    Òtúúrúpọ̀n - Memory Limits & Quotas                                       ║
║    Òtúrá   - Inter-Container Communication                                   ║
║    Ìrẹtẹ̀   - Garbage Collection & Cleanup                                    ║
║    Ọ̀ṣẹ́     - UI/Display Virtualization                                      ║
║    Òfún    - Capabilities & Permissions                                      ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys
import time
import threading
import tempfile
import shutil
import signal
import json
import hashlib
import subprocess
from pathlib import Path
from typing import Any, Dict, List, Optional, Callable
from dataclasses import dataclass, field
from enum import Enum

# Cross-platform compatibility
import platform

PLATFORM = platform.system()  # 'Windows', 'Darwin', 'Linux'

try:
    import resource
    HAS_RESOURCE = True
except ImportError:
    HAS_RESOURCE = False  # Windows

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False


def get_temp_dir() -> Path:
    """Get appropriate temp directory for platform."""
    if PLATFORM == 'Windows':
        return Path(os.environ.get('TEMP', tempfile.gettempdir()))
    return Path('/tmp')


def kill_process_tree(pid: int, force: bool = False):
    """Kill process and all children - cross-platform."""
    try:
        if HAS_PSUTIL:
            import psutil
            parent = psutil.Process(pid)
            for child in parent.children(recursive=True):
                if force:
                    child.kill()
                else:
                    child.terminate()
            if force:
                parent.kill()
            else:
                parent.terminate()
        elif PLATFORM == 'Windows':
            subprocess.run(['taskkill', '/F', '/T', '/PID', str(pid)], 
                         capture_output=True)
        else:
            os.kill(pid, signal.SIGKILL if force else signal.SIGTERM)
    except Exception:
        pass


# =============================================================================
# OGBÈ - CONTAINER LIFECYCLE
# =============================================================================

class ContainerState(Enum):
    """Container states following lifecycle."""
    CREATED = "created"
    RUNNING = "running"
    PAUSED = "paused"
    STOPPED = "stopped"
    EXITED = "exited"
    DESTROYED = "destroyed"


@dataclass
class OgbeConfig:
    """Ogbè - Container initialization configuration."""
    name: str
    image: str = "ifa:base"
    command: List[str] = field(default_factory=list)
    environment: Dict[str, str] = field(default_factory=dict)
    working_dir: str = "/workspace"
    user: str = "ifa"
    auto_remove: bool = True


class OgbeLifecycle:
    """Ogbè - Container lifecycle manager."""
    
    def __init__(self, config: OgbeConfig):
        self.config = config
        self.state = ContainerState.CREATED
        self.container_id = self._generate_id()
        self.created_at = time.time()
        self.started_at: Optional[float] = None
        self.stopped_at: Optional[float] = None
        
    def _generate_id(self) -> str:
        """Generate unique container ID."""
        data = f"{self.config.name}{time.time()}".encode()
        return hashlib.sha256(data).hexdigest()[:12]
    
    def transition(self, new_state: ContainerState) -> bool:
        """Transition to new state with validation."""
        valid_transitions = {
            ContainerState.CREATED: [ContainerState.RUNNING],
            ContainerState.RUNNING: [ContainerState.PAUSED, ContainerState.STOPPED],
            ContainerState.PAUSED: [ContainerState.RUNNING, ContainerState.STOPPED],
            ContainerState.STOPPED: [ContainerState.EXITED],
            ContainerState.EXITED: [ContainerState.DESTROYED],
        }
        
        if new_state in valid_transitions.get(self.state, []):
            print(f"[Ogbè] {self.config.name}: {self.state.value} → {new_state.value}")
            self.state = new_state
            return True
        
        print(f"[Ogbè] Invalid transition: {self.state.value} → {new_state.value}")
        return False


# =============================================================================
# ỌYẸKÚ - CONTAINER TERMINATION
# =============================================================================

class OyekuTerminator:
    """Ọ̀yẹ̀kú - Container termination and cleanup."""
    
    @staticmethod
    def terminate(container: 'IgbaleContainer', force: bool = False) -> bool:
        """Terminate container gracefully or forcefully."""
        if container.lifecycle.state == ContainerState.RUNNING:
            if force:
                print(f"[Ọ̀yẹ̀kú] Force killing container {container.name}")
                if container.process:
                    container.process.kill()
            else:
                print(f"[Ọ̀yẹ̀kú] Gracefully stopping container {container.name}")
                if container.process:
                    container.process.terminate()
                    try:
                        container.process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        container.process.kill()
            
            container.lifecycle.transition(ContainerState.STOPPED)
            container.lifecycle.stopped_at = time.time()
            return True
        
        return False
    
    @staticmethod
    def cleanup(container: 'IgbaleContainer') -> bool:
        """Clean up all container resources."""
        print(f"[Ọ̀yẹ̀kú] Cleaning up container {container.name}")
        
        # Remove virtual filesystem
        if container.odi_fs and container.odi_fs.root_path.exists():
            shutil.rmtree(container.odi_fs.root_path, ignore_errors=True)
        
        # Clear logs
        container.irosu_logger.clear()
        
        container.lifecycle.transition(ContainerState.DESTROYED)
        return True


# =============================================================================
# ÌWÒRÌ - TIME LIMITS & MONITORING
# =============================================================================

class IworiMonitor:
    """Ìwòrì - Time limits and resource monitoring."""
    
    def __init__(self, max_runtime: float = 300.0, check_interval: float = 1.0):
        self.max_runtime = max_runtime  # seconds
        self.check_interval = check_interval
        self.start_time: Optional[float] = None
        self.watchdog_thread: Optional[threading.Thread] = None
        self.running = False
        
    def start(self, container: 'IgbaleContainer'):
        """Start monitoring container."""
        self.start_time = time.time()
        self.running = True
        
        def watchdog():
            while self.running:
                elapsed = time.time() - self.start_time
                
                if elapsed > self.max_runtime:
                    print(f"[Ìwòrì] Container {container.name} exceeded time limit")
                    OyekuTerminator.terminate(container, force=True)
                    break
                
                time.sleep(self.check_interval)
        
        self.watchdog_thread = threading.Thread(target=watchdog, daemon=True)
        self.watchdog_thread.start()
    
    def stop(self):
        """Stop monitoring."""
        self.running = False


# =============================================================================
# ÒDÍ - FILESYSTEM ISOLATION
# =============================================================================

class OdiVirtualFS:
    """Òdí - Virtual filesystem with path isolation."""
    
    def __init__(self, container_name: str):
        self.container_name = container_name
        self.root_path = Path(tempfile.mkdtemp(prefix=f"ifa_container_{container_name}_"))
        self.max_file_size = 10 * 1024 * 1024  # 10MB
        self.max_total_size = 100 * 1024 * 1024  # 100MB
        self._setup_structure()
    
    def _setup_structure(self):
        """Create standard directory structure."""
        dirs = ['workspace', 'tmp', 'logs', 'data']
        for d in dirs:
            (self.root_path / d).mkdir(exist_ok=True)
    
    def _validate_path(self, path: str) -> Path:
        """Validate path is within container."""
        abs_path = (self.root_path / path).resolve()
        if not str(abs_path).startswith(str(self.root_path)):
            raise PermissionError(f"Path outside container: {path}")
        return abs_path
    
    def _check_quota(self) -> bool:
        """Check if total size is within quota."""
        total = sum(f.stat().st_size for f in self.root_path.rglob('*') if f.is_file())
        return total < self.max_total_size
    
    def read_file(self, path: str) -> str:
        """Read file from virtual filesystem."""
        safe_path = self._validate_path(path)
        if not safe_path.exists():
            raise FileNotFoundError(f"File not found: {path}")
        
        if safe_path.stat().st_size > self.max_file_size:
            raise ValueError(f"File too large: {path}")
        
        return safe_path.read_text(encoding='utf-8')
    
    def write_file(self, path: str, content: str) -> bool:
        """Write file to virtual filesystem."""
        if len(content) > self.max_file_size:
            raise ValueError("Content too large")
        
        if not self._check_quota():
            raise ValueError("Container storage quota exceeded")
        
        safe_path = self._validate_path(path)
        safe_path.parent.mkdir(parents=True, exist_ok=True)
        safe_path.write_text(content, encoding='utf-8')
        return True
    
    def list_dir(self, path: str = ".") -> List[str]:
        """List directory contents."""
        safe_path = self._validate_path(path)
        if not safe_path.is_dir():
            raise NotADirectoryError(f"Not a directory: {path}")
        return [f.name for f in safe_path.iterdir()]


# =============================================================================
# ÌROSÙ - I/O CAPTURE & LOGGING
# =============================================================================

class IrosuLogger:
    """Ìrosù - Capture and log container I/O."""
    
    def __init__(self, max_log_size: int = 1024 * 1024):  # 1MB
        self.stdout_buffer: List[str] = []
        self.stderr_buffer: List[str] = []
        self.max_log_size = max_log_size
        self.current_size = 0
    
    def log_stdout(self, message: str):
        """Log stdout message."""
        if self.current_size < self.max_log_size:
            self.stdout_buffer.append(message)
            self.current_size += len(message)
    
    def log_stderr(self, message: str):
        """Log stderr message."""
        if self.current_size < self.max_log_size:
            self.stderr_buffer.append(message)
            self.current_size += len(message)
    
    def get_logs(self, stdout: bool = True, stderr: bool = True) -> str:
        """Get container logs."""
        logs = []
        if stdout:
            logs.extend(f"[OUT] {line}" for line in self.stdout_buffer)
        if stderr:
            logs.extend(f"[ERR] {line}" for line in self.stderr_buffer)
        return "\n".join(logs)
    
    def clear(self):
        """Clear all logs."""
        self.stdout_buffer.clear()
        self.stderr_buffer.clear()
        self.current_size = 0


# =============================================================================
# ỌWỌNRÍN - ENTROPY ISOLATION
# =============================================================================

class OwonrinEntropy:
    """Ọ̀wọ́nrín - Isolated random number generation."""
    
    def __init__(self, seed: Optional[int] = None):
        self.seed = seed or int(time.time() * 1000) % (2**32)
        self._state = self.seed
    
    def random_int(self, min_val: int = 0, max_val: int = 100) -> int:
        """Isolated random integer (LCG)."""
        self._state = (1103515245 * self._state + 12345) % (2**31)
        return min_val + (self._state % (max_val - min_val + 1))


# =============================================================================
# ỌBÀRÀ - CPU LIMITS
# =============================================================================

class ObaraCPULimiter:
    """Ọ̀bàrà - CPU resource limits."""
    
    def __init__(self, cpu_quota: float = 0.5):
        self.cpu_quota = cpu_quota
    
    def apply(self, pid: int) -> bool:
        """Apply CPU limits to process."""
        if not HAS_RESOURCE:
            print("[Ọ̀bàrà] CPU limits not available on Windows")
            return False
        
        try:
            max_cpu_time = 60  # 60 seconds of CPU time
            resource.prlimit(pid, resource.RLIMIT_CPU, (max_cpu_time, max_cpu_time))
            print(f"[Ọ̀bàrà] CPU limit applied to PID {pid}")
            return True
        except Exception as e:
            print(f"[Ọ̀bàrà] Failed to apply CPU limits: {e}")
            return False


# =============================================================================
# ỌKÀNRÀN - SECURITY VIOLATIONS
# =============================================================================

class OkanranSecurityError(Exception):
    """Security violation detected."""
    pass


class OkanranSecurityMonitor:
    """Ọ̀kànràn - Security violation detection."""
    
    def __init__(self):
        self.violations: List[Dict] = []
    
    def check_syscall(self, syscall: str) -> bool:
        """Check if syscall is allowed."""
        blocked = {
            'execve', 'fork', 'clone', 'setuid', 'setgid',
            'mount', 'umount', 'reboot', 'swapon', 'swapoff'
        }
        
        if syscall in blocked:
            self.violations.append({
                'type': 'blocked_syscall',
                'syscall': syscall,
                'time': time.time()
            })
            raise OkanranSecurityError(f"Blocked syscall: {syscall}")
        
        return True
    
    def check_network_access(self, host: str, port: int) -> bool:
        """Check if network access is allowed."""
        blocked_hosts = ['127.0.0.1', 'localhost', '169.254.169.254']
        if host in blocked_hosts:
            self.violations.append({
                'type': 'blocked_network',
                'host': host,
                'port': port,
                'time': time.time()
            })
            raise OkanranSecurityError(f"Blocked network access: {host}:{port}")
        return True


# =============================================================================
# ÒGÚNDÁ - PROCESS CONTROL
# =============================================================================

class OgundaProcessController:
    """Ògúndá - Process control and limits."""
    
    def __init__(self, max_processes: int = 10):
        self.max_processes = max_processes
        self.process_count = 0
    
    def can_fork(self) -> bool:
        """Check if process can fork."""
        return self.process_count < self.max_processes
    
    def apply_limits(self, pid: int):
        """Apply process limits."""
        if not HAS_RESOURCE:
            print("[Ògúndá] Process limits not available on Windows")
            return
        
        try:
            resource.prlimit(pid, resource.RLIMIT_NPROC, 
                           (self.max_processes, self.max_processes))
            print(f"[Ògúndá] Process limits applied to PID {pid}")
        except Exception as e:
            print(f"[Ògúndá] Failed to apply limits: {e}")


# =============================================================================
# ỌSÁ - NETWORK NAMESPACE
# =============================================================================

class OsaNetworkNamespace:
    """Ọ̀sá - Network isolation."""
    
    def __init__(self, allow_network: bool = False):
        self.allow_network = allow_network
        self.allowed_hosts: List[str] = []
        self.allowed_ports: List[int] = []
    
    def allow_host(self, host: str, port: int):
        """Allow access to specific host:port."""
        self.allowed_hosts.append(host)
        self.allowed_ports.append(port)
    
    def check_access(self, host: str, port: int) -> bool:
        """Check if network access is allowed."""
        if not self.allow_network:
            return False
        if not self.allowed_hosts:
            return False
        return host in self.allowed_hosts and port in self.allowed_ports


# =============================================================================
# ÌKÁ - DATA SANITIZATION
# =============================================================================

class IkaSanitizer:
    """Ìká - String and data sanitization."""
    
    @staticmethod
    def sanitize_filename(filename: str) -> str:
        """Sanitize filename."""
        dangerous = set('/\\<>:"|?*\0')
        sanitized = ''.join(c if c not in dangerous else '_' for c in filename)
        return sanitized[:255]
    
    @staticmethod
    def sanitize_output(text: str) -> str:
        """Sanitize output text."""
        return ''.join(c for c in text if c.isprintable() or c in '\n\t')


# =============================================================================
# ÒTÚÚRÚPỌ̀N - MEMORY LIMITS
# =============================================================================

class OturuponMemoryLimiter:
    """Òtúúrúpọ̀n - Memory limits and quotas."""
    
    def __init__(self, max_memory: int = 100 * 1024 * 1024):  # 100MB
        self.max_memory = max_memory
    
    def apply(self, pid: int) -> bool:
        """Apply memory limits to process."""
        if not HAS_RESOURCE:
            print("[Òtúúrúpọ̀n] Memory limits not available on Windows")
            return False
        
        try:
            resource.prlimit(pid, resource.RLIMIT_AS, 
                           (self.max_memory, self.max_memory))
            print(f"[Òtúúrúpọ̀n] Memory limit applied: {self.max_memory / 1024 / 1024}MB")
            return True
        except Exception as e:
            print(f"[Òtúúrúpọ̀n] Failed to apply memory limits: {e}")
            return False


# =============================================================================
# ÒTÚRÁ - INTER-CONTAINER COMMUNICATION
# =============================================================================

class OturaICC:
    """Òtúrá - Inter-container communication."""
    
    def __init__(self):
        self.channels: Dict[str, List[Any]] = {}
        self._lock = threading.Lock()
    
    def create_channel(self, name: str):
        """Create communication channel."""
        with self._lock:
            self.channels[name] = []
    
    def send(self, channel: str, message: Any):
        """Send message to channel."""
        with self._lock:
            if channel in self.channels:
                self.channels[channel].append(message)
    
    def receive(self, channel: str) -> Optional[Any]:
        """Receive message from channel."""
        with self._lock:
            if channel in self.channels and self.channels[channel]:
                return self.channels[channel].pop(0)
        return None


# =============================================================================
# ÌRẸTẸ̀ - GARBAGE COLLECTION
# =============================================================================

class IreteGarbageCollector:
    """Ìrẹtẹ̀ - Container garbage collection."""
    
    @staticmethod
    def collect(containers: List['IgbaleContainer']):
        """Collect and remove exited containers."""
        collected = []
        for container in containers[:]:
            if container.lifecycle.state == ContainerState.EXITED:
                if container.lifecycle.config.auto_remove:
                    OyekuTerminator.cleanup(container)
                    collected.append(container.name)
                    containers.remove(container)
        
        if collected:
            print(f"[Ìrẹtẹ̀] Collected containers: {', '.join(collected)}")


# =============================================================================
# ỌṢẸ́ - DISPLAY VIRTUALIZATION
# =============================================================================

class OseDisplayVirtualizer:
    """Ọ̀ṣẹ́ - Virtual display for containers."""
    
    def __init__(self, width: int = 80, height: int = 24):
        self.width = width
        self.height = height
        self.buffer = [[' ' for _ in range(width)] for _ in range(height)]
    
    def write(self, x: int, y: int, text: str):
        """Write text to virtual display."""
        if 0 <= y < self.height:
            for i, char in enumerate(text):
                if 0 <= x + i < self.width:
                    self.buffer[y][x + i] = char
    
    def render(self) -> str:
        """Render display buffer."""
        return '\n'.join(''.join(row) for row in self.buffer)


# =============================================================================
# ÒFÚN - CAPABILITIES & PERMISSIONS
# =============================================================================

@dataclass
class OfunCapabilities:
    """Òfún - Container capabilities."""
    can_network: bool = False
    can_exec: bool = False
    can_mount: bool = False
    can_raw_sockets: bool = False
    can_sys_admin: bool = False
    max_open_files: int = 100
    max_file_size: int = 10 * 1024 * 1024


# =============================================================================
# ÌGBÁLẸ̀ CONTAINER - MAIN CLASS
# =============================================================================

class IgbaleContainer:
    """
    Ìgbálẹ̀ - The Sacred Ground
    Main container class integrating all 16 Odù domains.
    """
    
    def __init__(self, config: OgbeConfig, capabilities: Optional[OfunCapabilities] = None):
        # Ogbè - Lifecycle
        self.lifecycle = OgbeLifecycle(config)
        self.name = config.name
        
        # Òdí - Filesystem
        self.odi_fs = OdiVirtualFS(self.name)
        
        # Ìrosù - Logging
        self.irosu_logger = IrosuLogger()
        
        # Ìwòrì - Monitoring
        self.iwori_monitor = IworiMonitor()
        
        # Ọ̀wọ́nrín - Entropy
        self.owonrin_entropy = OwonrinEntropy()
        
        # Ọ̀bàrà - CPU limits
        self.obara_cpu = ObaraCPULimiter()
        
        # Ọ̀kànràn - Security
        self.okanran_security = OkanranSecurityMonitor()
        
        # Ògúndá - Process control
        self.ogunda_process = OgundaProcessController()
        
        # Ọ̀sá - Network
        self.osa_network = OsaNetworkNamespace()
        
        # Òtúúrúpọ̀n - Memory
        self.oturupon_memory = OturuponMemoryLimiter()
        
        # Ọ̀ṣẹ́ - Display
        self.ose_display = OseDisplayVirtualizer()
        
        # Òfún - Capabilities
        self.ofun_capabilities = capabilities or OfunCapabilities()
        
        # Process reference
        self.process: Optional[subprocess.Popen] = None
    
    def start(self) -> bool:
        """Start the container."""
        if not self.lifecycle.transition(ContainerState.RUNNING):
            return False
        
        self.lifecycle.started_at = time.time()
        self.iwori_monitor.start(self)
        
        if self.lifecycle.config.command:
            try:
                self.process = subprocess.Popen(
                    self.lifecycle.config.command,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    cwd=str(self.odi_fs.root_path / 'workspace'),
                    env=self.lifecycle.config.environment,
                    text=True
                )
                
                if self.process.pid:
                    self.obara_cpu.apply(self.process.pid)
                    self.oturupon_memory.apply(self.process.pid)
                    self.ogunda_process.apply_limits(self.process.pid)
                
            except Exception as e:
                print(f"[Error] Failed to start container: {e}")
                return False
        
        print(f"[Ìgbálẹ̀] Container {self.name} started (ID: {self.lifecycle.container_id})")
        return True
    
    def stop(self, force: bool = False) -> bool:
        """Stop the container."""
        self.iwori_monitor.stop()
        return OyekuTerminator.terminate(self, force)
    
    def logs(self) -> str:
        """Get container logs."""
        return self.irosu_logger.get_logs()
    
    def exec(self, command: List[str]) -> str:
        """Execute command in running container."""
        if self.lifecycle.state != ContainerState.RUNNING:
            return "Error: Container not running"
        
        if not self.ofun_capabilities.can_exec:
            return "Error: exec capability not granted"
        
        try:
            result = subprocess.run(
                command,
                cwd=str(self.odi_fs.root_path / 'workspace'),
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout
        except Exception as e:
            return f"Error: {e}"
    
    def inspect(self) -> Dict:
        """Inspect container state."""
        return {
            'Name': self.name,
            'ID': self.lifecycle.container_id,
            'State': self.lifecycle.state.value,
            'Created': self.lifecycle.created_at,
            'Started': self.lifecycle.started_at,
            'Stopped': self.lifecycle.stopped_at,
            'RootFS': str(self.odi_fs.root_path),
            'Capabilities': {
                'Network': self.ofun_capabilities.can_network,
                'Exec': self.ofun_capabilities.can_exec,
            }
        }


# =============================================================================
# CONTAINER RUNTIME
# =============================================================================

class IgbaleRuntime:
    """Container runtime manager."""
    
    def __init__(self):
        self.containers: List[IgbaleContainer] = []
        self.otura_icc = OturaICC()
    
    def create(self, config: OgbeConfig, capabilities: Optional[OfunCapabilities] = None) -> IgbaleContainer:
        """Create a new container."""
        container = IgbaleContainer(config, capabilities)
        self.containers.append(container)
        return container
    
    def list(self) -> List[Dict]:
        """List all containers."""
        return [c.inspect() for c in self.containers]
    
    def get(self, name: str) -> Optional[IgbaleContainer]:
        """Get container by name."""
        for c in self.containers:
            if c.name == name:
                return c
        return None
    
    def gc(self):
        """Run garbage collection."""
        IreteGarbageCollector.collect(self.containers)


# =============================================================================
# DEMO
# =============================================================================

def demo():
    """Demonstration of the Ìgbálẹ̀ sandbox system."""
    print("""
╔══════════════════════════════════════════════════════════════╗
║              ÌGBÁLẸ̀ - THE SACRED GROUND                     ║
║           Ifá Sandbox Container System Demo                  ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    runtime = IgbaleRuntime()
    
    # Example: Simple container
    print("\n=== Simple Container ===")
    config = OgbeConfig(name="test-container")
    container = runtime.create(config)
    container.start()
    
    # Write to virtual filesystem
    container.odi_fs.write_file("workspace/test.txt", "Hello Ifá!")
    content = container.odi_fs.read_file("workspace/test.txt")
    print(f"File content: {content}")
    
    # List containers
    print("\n=== Container List ===")
    for info in runtime.list():
        print(f"  {info['Name']}: {info['State']}")
    
    # Stop and clean
    container.stop()
    runtime.gc()
    
    print("\n[Ìgbálẹ̀] Demo complete!")


# =============================================================================
# INTERPRETER INTEGRATION
# =============================================================================

def run_sandboxed(code: str, timeout: float = 30.0, **kwargs) -> Dict:
    """
    Run Ifá code in a sandboxed environment.
    
    Args:
        code: Ifá source code to execute
        timeout: Maximum runtime in seconds
        **kwargs: Additional config options
    
    Returns:
        Dict with success, stdout, stderr, duration, violations
    """
    config = OgbeConfig(
        name=f"ifa_run_{int(time.time())}",
        auto_remove=True,
        **{k: v for k, v in kwargs.items() if hasattr(OgbeConfig, k)}
    )
    
    runtime = IgbaleRuntime()
    container = runtime.create(config)
    
    # Write code to virtual filesystem
    code_path = container.odi_fs.root_path / 'workspace' / 'script.ifa'
    code_path.write_text(code, encoding='utf-8')
    
    # Start container with Ifá interpreter
    container.lifecycle.config.command = [
        sys.executable, '-m', 'src.cli', 'run', str(code_path)
    ]
    
    container.iwori_monitor.max_runtime = timeout
    container.start()
    
    # Wait for completion
    stdout, stderr = "", ""
    if container.process:
        try:
            stdout, stderr = container.process.communicate(timeout=timeout)
            exit_code = container.process.returncode
        except subprocess.TimeoutExpired:
            container.stop(force=True)
            exit_code = -1
            stderr = "Execution timed out"
    else:
        exit_code = 0
    
    container.stop()
    
    result = {
        'success': exit_code == 0,
        'exit_code': exit_code,
        'stdout': stdout,
        'stderr': stderr,
        'duration': time.time() - container.lifecycle.started_at if container.lifecycle.started_at else 0,
        'violations': container.okanran_security.violations
    }
    
    runtime.gc()
    return result


def create_sandbox(name: str = "default", **kwargs) -> IgbaleContainer:
    """Create a sandbox instance for manual control."""
    config = OgbeConfig(name=name, **kwargs)
    runtime = IgbaleRuntime()
    return runtime.create(config)


if __name__ == "__main__":
    demo()


# Formalizing the Ifá-Lang Closed Effect Algebra

> [!WARNING]
> **Aspirational Design — The Effect Algebra is Not Completely Closed at the VM Level**
> As of June 2026, the Ifá-Lang VM has 7 side-effectful opcodes that operate outside the formal 16-domain algebra without proper attribution:
> 1. `Print` and `PrintRaw` (stdout)
> 2. `Input` (stdin)
> 3. `Import` (dynamic loading/module execution)
> 4. `Yield` (control flow yielding)
> 5. `EpochBegin` and `EpochEnd` (Ẹbọ epoch memory lifecycle management)
> 6. `Store8`–`Store64` / `Load8`–`Load64` (MMIO hardware register operations)
> While statically audited via Babalawo, these VM operations bypass the `CallOdu`/`CallOduFast` index routing checks. The `OduDomain::classify_effect` function has been introduced to resolve this runtime/static discrepancy.

This document establishes the formal mathematical model, semantics, and proofs for Ifá-Lang's domain-indexed effect taxonomy. By indexing all platform-level operations into a closed set of 16 principal Odù, Ifá-Lang enables static auditability and monotonic resource/capability attenuation.

---

## 1. Mathematical Formulation

Let $\Sigma$ represent the state of the execution environment (e.g., memory, filesystem, network, hardware peripherals). A program execution is a sequence of state transformations:

$$\sigma_0 \xrightarrow{\alpha_1} \sigma_1 \xrightarrow{\alpha_2} \dots \xrightarrow{\alpha_n} \sigma_n$$

where each transition $\alpha_i$ represents an action. We define the universe of all possible side-effecting actions as $\mathcal{A}$.

### Definition 1: The Effect Partitioning

We define a partition of $\mathcal{A}$ into $N = 16$ disjoint subsets corresponding to the 16 principal Odù domains:

$$\mathcal{A} = \biguplus_{d=0}^{15} \mathcal{D}_d \cup \mathcal{A}_{\text{pure}}$$

where:
*   $\mathcal{D}_d$ is the set of actions associated with the Odù domain of ID $d$.
*   $\mathcal{A}_{\text{pure}}$ represents pure computational actions that do not modify or query the external environment $\Sigma$.
*   For all $i \neq j$, $\mathcal{D}_i \cap \mathcal{D}_j = \emptyset$ (Disjointness).

### Definition 2: The Bytecode Homomorphism

Let $\mathcal{I}$ be the set of all bytecode instructions in the Ifá-Lang ISA. We define an effect classification function $\chi : \mathcal{I} \to \{0, \dots, 15, \bot\}$:

$$\chi(inst) = \begin{cases} 
      d & \text{if } inst = \mathtt{CallOdu}(d, m) \text{ or } inst = \mathtt{CallOduFast}(d, m) \\
      \bot & \text{otherwise}
   \end{cases}$$

This mapping guarantees that the side-effect category of *any* instruction is decidable in $O(1)$ time by examining only the opcode and its immediate arguments.

---

## 2. Capability Algebra and Monotonic Attenuation

The capability model of Ifá-Lang is governed by the Ọ̀fún capability set. Let $\mathcal{C}$ be the set of all possible capabilities (e.g., `ReadFiles`, `Network`, `Crypto`).

A program's capability environment is a tuple $\langle G, S \rangle$ where:
*   $G \subseteq \mathcal{C}$ is the set of **Granted** capabilities.
*   $S \subseteq \mathcal{C}$ is the set of **Sacrificed** (permanently revoked) capabilities.

### Axiom 1: Monotonic Sacrifice (The Ẹbọ Rule)
For any execution trace, once a capability $c \in \mathcal{C}$ is added to the sacrificed set $S$, it can never be removed:

$$\forall t_1 < t_2, \quad S(t_1) \subseteq S(t_2)$$

Furthermore, a capability $c$ is effectively permitted at time $t$ if and only if:

$$\text{Permitted}(c, t) \iff \Big( \exists g \in G(t). \ g \sqsupseteq c \Big) \land \Big( \forall s \in S(t). \ s \not\sqsupseteq c \Big)$$

where $a \sqsupseteq b$ denotes that capability $a$ subsumes capability $b$ (e.g., `ReadFiles(root="/")` subsumes `ReadFiles(root="/tmp")`).

---

## 3. Completeness Proof of the 16-Domain Taxonomy

To prove that the 16-domain taxonomy is complete, we must map every class of OS system call and hardware interaction to exactly one principal Odù:

| ID | Odù | Ifá Pattern | Category | OS/Hardware Map |
|----|-----|-------------|----------|-----------------|
| 0 | **Ọ̀gbè** | `1111` | Lifecycle | Program entry, CLI args, process control |
| 1 | **Ọ̀yẹ̀kú** | `0000` | Termination | Process exit, thread sleep, yielding |
| 2 | **Ìwòrì** | `0110` | Time | System clock, timers, epoch retrieval |
| 3 | **Òdí** | `1001` | Storage | Filesystem descriptors, block I/O, database mutations |
| 4 | **Ìrosù** | `1100` | Console/Audio | Standard streams (stdout, stderr), audio output |
| 5 | **Ọ̀wọ́nrín** | `0011` | Entropy | Kernel CSPRNG, random seeding |
| 6 | **Ọ̀bàrà** | `1000` | Math (Pure) | Basic arithmetic (add, multiply) |
| 7 | **Ọ̀kànràn** | `0001` | Diagnostics | Exceptions, panic unwinding, assertion failures |
| 8 | **Ògúndá** | `1110` | Arrays | Dynamic memory allocation, heap arrays |
| 9 | **Ọ̀sá** | `0111` | Concurrency | Thread spawn, IPC channels, async polling |
| 10| **Ìká** | `0100` | Strings | String allocations, format buffers |
| 11| **Òtúúrúpọ̀n**| `0010` | Math (Pure) | Advanced math (subtract, divide, modulo) |
| 12| **Òtúrá** | `1011` | Networking | Network sockets, TCP/UDP packets, HTTP client |
| 13| **Ìrẹtẹ̀** | `1101` | Cryptography | Hashing engines, symmetric key encryption |
| 14| **Ọ̀ṣẹ́** | `1010` | Presentation | Graphics framebuffers, terminal cursor control, TUI |
| 15| **Òfún** | `0101` | Permissions | Reflection, capability queries, dynamic revocation |

### Disjointness Theorem

Let $f_c$ be a system call or hardware interrupt. There exists a unique mapping function $\mu : \text{Syscalls} \to \{0, \dots, 15\}$ such that:

$$\forall s \in \text{Syscalls}, \quad |\mu(s)| = 1$$

*   **Proof Sketch**: 
    Suppose a system call like `sys_write` exists. If it writes to a file descriptor representing a block device or file, it maps to **Òdí** (3). If it writes to standard output, it maps to **Ìrosù** (4). If it writes to a network socket, it maps to **Òtúrá** (12). The target resource's descriptor type determines the domain mapping uniquely, ensuring disjointness at execution time.

---

## 4. Static Auditing and Verification

Because the effect system is reified at the bytecode level, an auditor program can scan a compiled `.ifab` binary to extract the exact list of required domains without execution:

```
Algorithm 1: Bytecode Effect Audit
Input: Bytecode program P
Output: Set of required domains R

R = {}
for instruction in P.instructions:
    if instruction.opcode == CallOdu or instruction.opcode == CallOduFast:
        R.add(instruction.domain_id)
return R
```

This guarantees that a compiled application can be statically verified to be sandboxed before it ever runs on the host VM or embedded device.

---

## 5. Engineering Realities, Limitations, and Leaks

While the closed effect algebra is an elegant mathematical representation, actual production execution introduces critical engineering leaks that violate strict containment:

### 5.1 The FFI/Bridge Escape Hatch
If a program is granted the `Ofun::Bridge` capability (mapping to FFI via Python, JS, or native C), the closed algebra is bypassed. The VM cannot trace, analyze, or restrict actions executed within foreign stack frames:

$$\forall a \in \mathcal{A}_{\text{FFI}}, \quad \chi(a) = \bot \quad \text{but} \quad a \text{ can mutate } \Sigma$$

FFI boundaries must be audited out-of-band; sandboxing is not mathematically absolute if FFI is permitted.

### 5.2 Syscall Multiplexing Complexity
Actual OS syscalls (e.g., `ioctl`, `fcntl`, `sys_write`) multiplex diverse side effects across uniform file descriptors. 
* A write operation to `/dev/null` vs a TCP socket vs an anonymous pipe behaves differently.
* Intercepting and mapping these requires runtime virtualization overhead at the VM-host interface, rather than a purely static $O(1)$ bytecode mapping.

### 5.3 The Tier 2 Domain Extension
The algebra is only closed with respect to the 16 traditional principal Odù (0–15). The introduction of Tier 2 infrastructure domains (`Cpu` (18), `Gpu` (19), `Storage` (20), `Sys` (29)) turns the system into an **extensible effect algebra**. Any platform expansion with new hardware drivers requires allocating new domain IDs, modifying the compiler, and expanding the VM dispatch table.

### 5.4 Dynamic Bytecode Execution
The static auditability theorem assumes the absence of self-modifying code, dynamic eval functions, or loading unverified bytecode over network interfaces. If the VM executes bytecode dynamically fetched at runtime, safety manifests must be validated dynamically during class-loading transitions.


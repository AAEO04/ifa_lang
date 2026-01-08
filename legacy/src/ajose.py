# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ÀJỌṢE - REACTIVE RELATIONSHIP ENGINE                      ║
║                    "The Bond That Responds"                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Àjọṣe (Relationship) - Pub/Sub pattern for reactive relationships.         ║
║  When entity state changes, related entities automatically respond.          ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import Any, Callable, Dict, List, Optional, Tuple
from dataclasses import dataclass, field
from collections import defaultdict
import weakref
import ast
import operator


# =============================================================================
# SAFE CONSTRAINT EVALUATOR (No eval!)
# =============================================================================

class SafeConstraintEvaluator:
    """
    Safe constraint evaluator using AST parsing.
    Only allows comparison operations - no arbitrary code execution.
    """
    
    # Whitelist of allowed attributes for security
    ALLOWED_ATTRS = {
        'balance', 'amount', 'value', 'status', 'count', 'name', 
        'old', 'new', 'id', 'type', 'size', 'length', 'index'
    }
    
    ALLOWED_OPS = {
        ast.Eq: operator.eq,
        ast.NotEq: operator.ne,
        ast.Lt: operator.lt,
        ast.LtE: operator.le,
        ast.Gt: operator.gt,
        ast.GtE: operator.ge,
    }
    
    def evaluate(self, constraint: str, source: Any, target: Any, context: Dict) -> bool:
        """Safely evaluate a constraint expression."""
        try:
            tree = ast.parse(constraint, mode='eval')
            return self._eval_node(tree.body, source, target, context)
        except:
            return True  # Default to allowing if can't parse
    
    def _eval_node(self, node, source, target, context) -> bool:
        if isinstance(node, ast.Compare):
            left = self._get_value(node.left, source, target, context)
            for op, right in zip(node.ops, node.comparators):
                right_val = self._get_value(right, source, target, context)
                if type(op) not in self.ALLOWED_OPS:
                    return True  # Unsupported op, allow by default
                if not self.ALLOWED_OPS[type(op)](left, right_val):
                    return False
                left = right_val
            return True
        elif isinstance(node, ast.BoolOp):
            if isinstance(node.op, ast.And):
                return all(self._eval_node(v, source, target, context) for v in node.values)
            elif isinstance(node.op, ast.Or):
                return any(self._eval_node(v, source, target, context) for v in node.values)
        elif isinstance(node, ast.Constant):
            return bool(node.value)
        return True
    
    def _get_value(self, node, source, target, context) -> Any:
        if isinstance(node, ast.Constant):
            return node.value
        elif isinstance(node, ast.Num):  # Python 3.7 compat
            return node.n
        elif isinstance(node, ast.Str):  # Python 3.7 compat
            return node.s
        elif isinstance(node, ast.Name):
            if node.id == 'source':
                return source
            elif node.id == 'target':
                return target
            elif node.id in context:
                return context[node.id]
            return None
        elif isinstance(node, ast.Attribute):
            # Security: Only allow whitelisted attributes
            if node.attr not in self.ALLOWED_ATTRS:
                return None  # Deny access to non-whitelisted attributes
            obj = self._get_value(node.value, source, target, context)
            if obj is not None:
                return getattr(obj, node.attr, None)
        return None


# =============================================================================
# DATA CLASSES
# =============================================================================

@dataclass
class Relationship:
    """Defines a relationship pattern between entities."""
    name: str
    source_type: str
    target_type: str
    bidirectional: bool = False
    constraints: List[str] = field(default_factory=list)


@dataclass  
class RelationshipInstance:
    """A specific relationship between two entity instances."""
    relationship: Relationship
    _source_ref: weakref.ref = field(repr=False)
    _target_ref: weakref.ref = field(repr=False)
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    @property
    def source(self) -> Optional[Any]:
        return self._source_ref() if self._source_ref else None
    
    @property
    def target(self) -> Optional[Any]:
        return self._target_ref() if self._target_ref else None


# =============================================================================
# ÀJỌṢE ENGINE
# =============================================================================

class AjosePredicateEngine:
    """
    The Àjọṣe Engine - Reactive Relationship Manager.
    
    Usage:
        engine = AjosePredicateEngine()
        engine.define("Transfer", "Wallet", "Wallet", bidirectional=True)
        
        @engine.when("Transfer")
        def on_transfer(source, target, context):
            source.balance -= context['amount']
            target.balance += context['amount']
        
        engine.link("Transfer", wallet1, wallet2, amount=30)
    """
    
    def __init__(self):
        self.relationships: Dict[str, Relationship] = {}
        self.instances: List[RelationshipInstance] = []
        self.subscribers: Dict[str, List[Callable]] = defaultdict(list)
        self._tracked_objects: Dict[int, Dict[str, Any]] = {}
        self._constraint_evaluator = SafeConstraintEvaluator()
    
    def define(self, name: str, source_type: str, target_type: str,
               bidirectional: bool = False, constraints: List[str] = None) -> Relationship:
        """Define a relationship pattern."""
        rel = Relationship(
            name=name, source_type=source_type, target_type=target_type,
            bidirectional=bidirectional, constraints=constraints or []
        )
        self.relationships[name] = rel
        arrow = "⟷" if bidirectional else "→"
        print(f"  [Àjọṣe] Defined: Àjọṣe({source_type} {arrow} {target_type})")
        return rel
    
    def when(self, relationship_name: str) -> Callable:
        """Decorator to subscribe to relationship events."""
        def decorator(fn: Callable) -> Callable:
            self.subscribers[relationship_name].append(fn)
            print(f"  [Àjọṣe] Subscribed: {fn.__name__} → {relationship_name}")
            return fn
        return decorator
    
    def subscribe(self, relationship_name: str, callback: Callable):
        """Programmatically subscribe to a relationship."""
        self.subscribers[relationship_name].append(callback)
    
    def link(self, relationship_name: str, source: Any, target: Any, 
             **context) -> RelationshipInstance:
        """Create a relationship instance and trigger callbacks."""
        if relationship_name not in self.relationships:
            raise ValueError(f"Unknown relationship: {relationship_name}")
        
        rel = self.relationships[relationship_name]
        
        # Check constraints using SAFE evaluator (no eval!)
        for constraint in rel.constraints:
            if not self._constraint_evaluator.evaluate(constraint, source, target, context):
                raise ValueError(f"Constraint failed: {constraint}")
        
        # Use weak references to prevent memory leaks
        # Handle non-weakreferenceable objects (strings, numbers, etc.)
        try:
            source_ref = weakref.ref(source)
        except TypeError:
            # Not weakreferenceable - use a lambda that returns the value
            source_ref = lambda: source
        
        try:
            target_ref = weakref.ref(target)
        except TypeError:
            # Not weakreferenceable - use a lambda that returns the value
            target_ref = lambda: target
        
        instance = RelationshipInstance(
            relationship=rel, 
            _source_ref=source_ref,
            _target_ref=target_ref, 
            metadata=context
        )
        self.instances.append(instance)
        
        # Periodically clean up dead refs (every 100 links)
        if len(self.instances) % 100 == 0:
            self.cleanup_dead_refs()
        
        self._notify(relationship_name, source, target, context)
        return instance
    
    def _notify(self, relationship_name: str, source: Any, target: Any, context: Dict):
        """Notify all subscribers of a relationship event."""
        for callback in self.subscribers.get(relationship_name, []):
            callback(source, target, context)
    
    def get_related(self, entity: Any, relationship_name: str = None) -> List[Any]:
        """Get all entities related to the given entity."""
        related = []
        dead_indices = []
        
        for i, inst in enumerate(self.instances):
            if relationship_name and inst.relationship.name != relationship_name:
                continue
            
            # Get live references
            source = inst.source
            target = inst.target
            
            # Mark dead instances for removal
            if source is None or target is None:
                dead_indices.append(i)
                continue
            
            if source is entity:
                related.append(target)
            elif inst.relationship.bidirectional and target is entity:
                related.append(source)
        
        # Clean up dead instances (reverse order to preserve indices)
        for i in reversed(dead_indices):
            del self.instances[i]
        
        return related
    
    def track(self, obj: Any):
        """Track an object for reactive updates."""
        obj_id = id(obj)
        self._tracked_objects[obj_id] = {k: getattr(obj, k, None) 
                                         for k in dir(obj) if not k.startswith('_')}
    
    def check_changes(self, obj: Any) -> Dict[str, Tuple[Any, Any]]:
        """Check for changes in a tracked object."""
        obj_id = id(obj)
        if obj_id not in self._tracked_objects:
            return {}
        
        changes = {}
        old_state = self._tracked_objects[obj_id]
        for attr in old_state:
            new_val = getattr(obj, attr, None)
            if old_state[attr] != new_val:
                changes[attr] = (old_state[attr], new_val)
                old_state[attr] = new_val
        return changes
    
    def cleanup_dead_refs(self):
        """Remove all instances with dead references."""
        self.instances = [inst for inst in self.instances 
                         if inst.source is not None and inst.target is not None]
    
    def __repr__(self):
        return f"AjosePredicateEngine({len(self.relationships)} relationships, {len(self.instances)} instances)"


__all__ = ['Relationship', 'RelationshipInstance', 'AjosePredicateEngine', 'SafeConstraintEvaluator']

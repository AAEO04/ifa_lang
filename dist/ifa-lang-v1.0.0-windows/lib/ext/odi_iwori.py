# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ÒDÍ_ÌWÒRÌ - Storage Analysis                              ║
║                    SQL / Queries / Indexing / Search                         ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Compound: Òdí (Storage/Seal) + Ìwòrì (Analysis/Reflection)                  ║
║  Meaning:  "Analyzing the Container"                                         ║
║  Opcode:   0x32 (Parent: 0x3, Child: 0x2)                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import sqlite3
from typing import List, Dict, Any, Optional


class OdiIwori:
    """Storage Analysis - SQL, Queries, Indexing"""
    
    _connection: Optional[sqlite3.Connection] = None
    _cursor: Optional[sqlite3.Cursor] = None
    
    @classmethod
    def ṣi(cls, path: str) -> bool:
        """
        ṣí = Open (Open database connection)
        
        Args:
            path: Path to SQLite database
            
        Returns:
            True if connection opened successfully
        """
        try:
            cls._connection = sqlite3.connect(path)
            cls._cursor = cls._connection.cursor()
            print(f"📂 [Òdí_Ìwòrì] Opened: {path}")
            return True
        except Exception as e:
            print(f"❌ [Òdí_Ìwòrì] Failed to open: {e}")
            return False
    
    @classmethod
    def bere(cls, query: str, params: tuple = ()) -> List[Dict[str, Any]]:
        """
        bẹ̀rẹ̀ = Ask/Query (Execute SQL SELECT)
        
        Args:
            query: SQL query string
            params: Query parameters
            
        Returns:
            List of result dictionaries
        """
        if not cls._cursor:
            return []
        
        try:
            cls._cursor.execute(query, params)
            columns = [desc[0] for desc in cls._cursor.description or []]
            results = []
            for row in cls._cursor.fetchall():
                results.append(dict(zip(columns, row)))
            return results
        except Exception as e:
            print(f"❌ [Òdí_Ìwòrì] Query failed: {e}")
            return []
    
    @classmethod
    def ṣe(cls, query: str, params: tuple = ()) -> int:
        """
        ṣe = Do/Execute (Execute SQL INSERT/UPDATE/DELETE)
        
        Args:
            query: SQL statement
            params: Query parameters
            
        Returns:
            Number of affected rows
        """
        if not cls._cursor or not cls._connection:
            return 0
        
        try:
            cls._cursor.execute(query, params)
            cls._connection.commit()
            return cls._cursor.rowcount
        except Exception as e:
            print(f"❌ [Òdí_Ìwòrì] Execute failed: {e}")
            return 0
    
    @classmethod
    def ka(cls, table: str) -> int:
        """
        kà = Count (Count rows in table)
        
        Args:
            table: Table name
            
        Returns:
            Row count
        """
        results = cls.bere(f"SELECT COUNT(*) as count FROM {table}")
        if results:
            return results[0].get('count', 0)
        return 0
    
    @classmethod
    def pa(cls) -> bool:
        """
        pa = Close (Close database connection)
        
        Returns:
            True if closed successfully
        """
        if cls._connection:
            cls._connection.close()
            cls._connection = None
            cls._cursor = None
            print("📁 [Òdí_Ìwòrì] Connection closed")
            return True
        return False


# Module-level functions for direct access
def si(path: str) -> bool:
    return OdiIwori.ṣi(path)

def bere(query: str, params: tuple = ()) -> List[Dict[str, Any]]:
    return OdiIwori.bere(query, params)

def se(query: str, params: tuple = ()) -> int:
    return OdiIwori.ṣe(query, params)

def ka(table: str) -> int:
    return OdiIwori.ka(table)

def pa() -> bool:
    return OdiIwori.pa()

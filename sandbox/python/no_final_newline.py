#!/usr/bin/env python3
"""File missing final newline."""

def process():
    """Process function."""
    data = [1, 2, 3, 4, 5]
    total = sum(data)
    return total

def validate(value):
    """Validate input."""
    return isinstance(value, int) and value > 0

class Processor:
    """Simple processor class."""
    
    def __init__(self, name):
        self.name = name
    
    def run(self):
        """Run processing."""
        return f"Processing {self.name}"

if __name__ == "__main__":
    p = Processor("test")
    print(p.run())
    print(process())
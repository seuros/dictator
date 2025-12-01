#!/usr/bin/env python3
"""File with trailing whitespace violations."""

import os
import sys

def hello_world():   
    """Say hello."""
    print("Hello, World!")  

def goodbye_world():
    """Say goodbye."""   
    print("Goodbye!")

class Example:
    """Example class with trailing spaces."""

    def __init__(self):
        self.value = 42    
        self.name = "test"

    def method(self):
        """A method with trailing space."""   
        return self.value

if __name__ == "__main__":
    ex = Example()
    print(ex.value)   

#!/usr/bin/env python3
"""File with mixed tabs and spaces (structural violation)."""

import sys
import os

def function_with_spaces():
    """This uses spaces for indentation."""
    value = 42
    result = value * 2
    return result

def function_with_tabs():
	"""This uses tabs for indentation."""
	value = 42
	result = value * 2
	return result

class MixedIndentClass:
    """Class mixing spaces and tabs."""
    
    def method_spaces(self):
        """Method using spaces."""
        x = 1
        y = 2
        return x + y
    
    def method_tabs(self):
	"""Method using tabs."""
	x = 1
	y = 2
	return x + y

if __name__ == "__main__":
    print(function_with_spaces())
    print(function_with_tabs())
    obj = MixedIndentClass()
    print(obj.method_spaces())
    print(obj.method_tabs())

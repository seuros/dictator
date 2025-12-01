#!/usr/bin/env python3
"""File with wrong import ordering."""

import requests
import os
import sys
from typing import Dict, List
import json
from collections import defaultdict

def fetch_data(url):
    """Fetch data from URL."""
    response = requests.get(url)
    return response.json()

def process_data(data):
    """Process data structure."""
    result = defaultdict(list)
    for item in data:
        result[item['type']].append(item)
    return result

def get_environment():
    """Get environment variables."""
    return {
        'path': os.getenv('PATH'),
        'home': os.getenv('HOME'),
        'user': os.getenv('USER')
    }

def serialize_result(data):
    """Convert to JSON."""
    return json.dumps(data)

if __name__ == "__main__":
    print("Environment loaded")

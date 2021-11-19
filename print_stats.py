import json
import sys
import math
import matplotlib
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MultipleLocator
import pandas as pd
from collections import defaultdict

ENTER = True
EXIT = False

ENTER_SYMBOL = "[+]"
EXIT_SYMBOL = "[-]"

# Convert seconds to microseconds
def to_micro(secs):
    return secs * float(1000000)

# Class representing either the entry or exit of a hostcall
class Event(object):
  """docstring for Event"""
  def __init__(self, name, timestamp, enter):
    self.name = name
    self.timestamp = timestamp 
    self.enter = enter

  def __repr__(self):
    rep = 'Event(' + self.name + ',' + str(self.timestamp) + ',' + str(self.enter) + ')'
    return rep

# [filenames] -> [Event]
def get_data(filenames):
    dataset = []
    for filename in filenames:
        data = {}
        file = open(filename, mode = 'r', encoding = 'utf-8-sig')
        lines = file.readlines()
        file.close()
        for line in lines:
            event = parse_event(line)
            if event != None:
                dataset.append(event)
    return dataset


# Example: 
# [2021-11-19T00:09:58.366356392Z TRACE veriwasi::wasm2c_frontend] [-] Exiting veriwasi_init = 0x558646d0d080
# lhs = [2021-11-19T00:09:58.366356392Z TRACE veriwasi::wasm2c_frontend]
# rhs = Exiting veriwasi_init = 0x558646d0d080
# name = veriwasi_init
# str_timestamp = 2021-11-19T00:09:58.366356392Z
# string -> Option<Event>
def parse_event(line):
    if ENTER_SYMBOL in line:
        lhs,rhs = line.split(ENTER_SYMBOL)
        enter = ENTER
    elif EXIT_SYMBOL in line:
        lhs,rhs = line.split(EXIT_SYMBOL)
        enter = EXIT
    else:
        return None
    name = rhs.split()[1]
    # Enter events at this stage are name(first_arg,
    if enter:
        name = name.split('(')[0]
    str_timestamp = lhs.split()[0].strip("[")
    timestamp = pd.to_datetime(str_timestamp, format='%Y-%m-%dT%H:%M:%S.%fZ')
    return Event(name, timestamp, enter)

# Transforms a list of events of the form (name, timestamp, enter/exit)
# to HostCallInfo of (name,duration) by matching 1 enter to 1 exit
# for now, it assumes these will be adjacent
# [Event] -> {name -> [duration]}
def compute_intervals(events):
    # first element might be an exit if we initialize logs inside this function
    if not events[0].enter:
        events = events[1:]
    # will probably end with an exit
    if events[-1].enter:
        events = events[:-1]

    intervals = defaultdict(list)
    num_events = len(events)
    for idx in range(0, num_events, 2):
        enter_event = events[idx]
        exit_event = events[idx+1]
        #cprint(enter_event, exit_event)
        
        assert(enter_event.enter)
        assert(not exit_event.enter)
        duration = exit_event.timestamp - enter_event.timestamp
        
        assert(enter_event.name == exit_event.name)
        name = enter_event.name 
        
        intervals[name].append(duration)
    return dict(intervals)



# {name -> duration} -> ()
# Prints average time of calls in nanoseconds
def print_stats(intervals):
    for name,interval in intervals.items():
        micro_interval = [to_micro(t).total_seconds() for t in interval]
        average = sum(micro_interval) / float(len(micro_interval))
        print(name, average)
    

def run(filenames):
    dataset = get_data(filenames)
    intervals = compute_intervals(dataset)
    print_stats(intervals)

def main():
    filenames = sys.argv[1:]
    run(filenames)

if __name__ == "__main__":
    main()

import matplotlib.pyplot as plt
import json
import sys
from typing import List
 
def lataa_tiedosto(tiedoston_nimi):
    try:
        with open(tiedoston_nimi, encoding='utf8') as tiedosto:
            jsontiedosto = json.load(tiedosto)
    except (FileNotFoundError, json.JSONDecodeError, UnboundLocalError):
        print("Tiedoston avaaminen ei onnistu.")
    return jsontiedosto
 
tiedos = lataa_tiedosto(sys.argv[1])
 
class data(object):
    def __init__(self, pressure_data: str):
        self.pressure_data = tiedos['pressure_data']
 
class Team(object):
    def __init__(self, data = List[tiedos]):
        self.data = data
 
data_n = data(pressure_data = tiedos['pressure_data'])
team = Team(data = [data_n])
for pressure in data_n.pressure_data:
    print(pressure)
 
 
json_data = json.dumps(team, default=lambda o: o.__dict__, indent=7)
#print(json_data)

import sys
from os import listdir
from os.path import isfile, join

folder = sys.argv[1]

tekst = """


##

"""
onlyfiles = [f for f in listdir(folder) if isfile(join(folder, f))]
nazwa = (
    "merged"
    + folder.strip().replace("\\", "").replace(".", "").replace("/", "")
    + ".txt"
)
print(onlyfiles)
with open(nazwa, "a", encoding="utf-8") as plik:
    for i in onlyfiles:
        if "txt" in i:
            print(i)
            with open(join(folder, i), "r", encoding="utf-8") as doodczytu:
                dane = doodczytu.read()
                plik.write(dane)
                plik.write(tekst)
        else:
            continue

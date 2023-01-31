import yaml
import csv
import sys

for file in sys.argv[1:]:
    with open(file) as csvfile:
        r = csv.DictReader(csvfile, dialect='excel')
        data = [x for x in r]
        name = file.split('.csv')[0].lower()
        with open(name + '.yml', "w") as out:
            out.write(yaml.dump({"dataset": [{"db": "tdvt", "collection": name, "docs": data}]}))


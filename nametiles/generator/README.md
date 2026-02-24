First run this on OSM PBF file to create an fgb file.
Then convert that to pmtiles. I used tippecanoe in the following way:
```bash
tippecanoe -z 10 -Z 10 -o out.pmtiles --drop-densest-as-needed dataset.fgb --read-parallel --force
```

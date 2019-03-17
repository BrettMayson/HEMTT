# Utilities

# translation

The translation utility will scan your project for `stringtable.xml` files and will tally up the keys. It will display a table with the key counts and the completion percentage. Supports all [Arma 3 Languages](https://community.bistudio.com/wiki/Stringtable.xml#Supported_languages).

`hemtt translation`
```
Total            2698
English          2698 100%
Italian          2677  99%
Polish           2677  99%
Japanese         2647  98%
Chinese          2609  97%
Chinesesimp      2601  96%
German           2529  94%
Korean           2471  92%
French           2451  91%
Russian          2178  81%
Czech            2085  77%
Portuguese       2067  77%
Spanish          2015  75%
Hungarian        1558  58%
Turkish             3   0%
```

# zip

The zip utility will zip the current release into a .zip file. The zip filename will be `{{name}}_{{version}}` unless a name is provided.

`hemtt zip` => `ace3_1.2.3.zip`
`hemtt zip release` => `release.zip`
`hemtt zip {{version}}` => `1.2.3.zip`


# convertproject
The convert project utility will convert the HEMTT project file from `hemtt.json -> hemtt.toml` or `hemtt.toml -> hemtt.json`.

`hemtt convertproject`

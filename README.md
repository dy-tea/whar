# whar - weird huffman archiver

Simple custom file archiver in blazingly fast rust (crab rocket fire).

### Usage
```
whar <command> <archive_name> [<file_names>...]
```

### Commands
```
a | archive : archive files
x | extract : extract files from archive
h | help : print the help message
```

### Examples
```
whar a files file1.jpg file2.md # Creates an archive named 'files.whar' containing the file1.jpg and file2.md
```
```
whar x files.whar # Extracts the the archive 'files.whar'
```

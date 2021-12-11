
import os
entries = os.scandir('.')
for entry in entries:
    try:
        print(entry.name.encode())
    except:
        print("unprintable")

    val = input("delete? ")
    if val == "y":
        if entry.is_dir(follow_symlinks=False):
            os.rmdir(entry.name)
        else:
            os.remove(entry)

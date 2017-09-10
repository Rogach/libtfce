#!/usr/bin/python3

import numpy
from array import array
import subprocess

subject_count = 10
data_length = 1000

# create random data arrays
A = numpy.random.uniform(size=(subject_count, data_length))
B = numpy.random.uniform(size=(subject_count, data_length))

# add artificial effect
B[:, 100:200] += 0.7

# check that python outputs data to binary files with expected byte count
assert array("B", [0]).itemsize == 1
assert array("I", [0]).itemsize == 4
assert array("d", [0]).itemsize == 8

# write input data to file
data_file = open("data.bin", "wb")
array("I", [subject_count]).tofile(data_file)
for s in range(subject_count):
    array("I", [data_length]).tofile(data_file)
    array("d", A[s]).tofile(data_file)
    array("I", [data_length]).tofile(data_file)
    array("d", B[s]).tofile(data_file)
data_file.close()

# call libtfce binary
subprocess.call([
    "target/release/libtfce",
    "--type", "1d",
    "--input-file", "data.bin",
    "--output-file", "result.bin",
    "--k", "0.666",
    "--e", "2.0",
    "--permutation-count", "1000"
])

# read result back
result_file = open("result.bin", "rb")
result_size = array("I", [])
result_size.fromfile(result_file, 1)
result = array("B", [])
result.fromfile(result_file, result_size[0])
result = result.tolist()
result_file.close()

# result list contains 1 where difference is significant
# and 0 where it is not
print(result)

#
/*

Copyright (c) 2017, Schizofreny s.r.o - info@schizofreny.com
All rights reserved.

See LICENSE.md file

*/

#include <vector>
#include <stdint.h>
#include <stdlib.h>
#include <iostream>
#include <memory>
#include "rust/cxx.h"

#ifndef MIDDLEOUT_H_
#define MIDDLEOUT_H_

namespace middleout {

std::unique_ptr<std::vector<char>> compressSimple(std::vector<int64_t>& data);

std::unique_ptr<std::vector<char>> compressSimple(std::vector<double>& data);

size_t compress(std::vector<int64_t>& data, std::vector<char>& output);

size_t compress(std::vector<double>& data, std::vector<char>& output);

void decompress(std::vector<char>& input, size_t itemsCount, std::vector<int64_t>& data);

void decompress(std::vector<char>& input, size_t itemsCount, std::vector<double>& data);

rust::Vec<uint8_t> compressInt(const rust::Vec<int64_t>& data);

rust::Vec<int64_t> decompressInt(const rust::Vec<uint8_t>& input, size_t inputElements);

rust::Vec<uint8_t> compressDouble(const rust::Vec<double>& data);

rust::Vec<double> decompressDouble(const rust::Vec<uint8_t>& input, size_t inputElements);

size_t maxCompressedSize(size_t count);

}  // end namespace middleout

#endif  // MIDDLEOUT_H_
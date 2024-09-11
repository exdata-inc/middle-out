/*

Copyright (c) 2017, Schizofreny s.r.o - info@schizofreny.com
All rights reserved.

See LICENSE.md file

*/

#include <vector>
#include <stdint.h>
#include <stdlib.h>
#include <memory>
#include "middleout.hpp"

#ifdef USE_AVX512
#include "avx512.hpp"
#define ALG_CLASS Avx52
#else
#include "scalar.hpp"
#define ALG_CLASS Scalar
#endif

namespace middleout {

std::unique_ptr<std::vector<char>> compressSimple(std::vector<int64_t>& data) {
	return ALG_CLASS<int64_t>::compressSimple(data);
}

std::unique_ptr<std::vector<char>> compressSimple(std::vector<double>& data) {
	return ALG_CLASS<double>::compressSimple(data);
}

size_t compress(std::vector<int64_t>& data, std::vector<char>& output) {
	return ALG_CLASS<int64_t>::compress(data, output);
}

size_t compress(std::vector<double>& data, std::vector<char>& output) {
	return ALG_CLASS<double>::compress(data, output);
}

void decompress(std::vector<char>& input, size_t inputElements, std::vector<int64_t>& data) {
	return ALG_CLASS<int64_t>::decompress(input, inputElements, data);
}

void decompress(std::vector<char>& input, size_t inputElements, std::vector<double>& data) {
	return ALG_CLASS<double>::decompress(input, inputElements, data);
}

rust::Vec<uint8_t> compressInt(const rust::Vec<int64_t>& data) {
    std::vector<int64_t> data_vec(data.begin(), data.end());
    std::unique_ptr<std::vector<char>> compressed_data_ptr = ALG_CLASS<int64_t>::compressSimple(data_vec);
    std::vector<char> compressed_data = std::move(*compressed_data_ptr); // moveで所有権を移動

    rust::Vec<uint8_t> result;
    result.reserve(compressed_data.size()); // Rustのベクタに容量を確保
    for (char byte : compressed_data) {
        result.push_back(static_cast<uint8_t>(byte)); // 要素を一つずつ追加
    }
    return result;
}

rust::Vec<uint8_t> compressDouble(const rust::Vec<double>& data) {
    std::vector<double> data_vec(data.begin(), data.end());
    std::unique_ptr<std::vector<char>> compressed_data_ptr = ALG_CLASS<double>::compressSimple(data_vec);
    std::vector<char> compressed_data = std::move(*compressed_data_ptr); // moveで所有権を移動

    rust::Vec<uint8_t> result;
    result.reserve(compressed_data.size()); // Rustのベクタに容量を確保
    for (char byte : compressed_data) {
        result.push_back(static_cast<uint8_t>(byte)); // 要素を一つずつ追加
    }
    return result;
}

rust::Vec<int64_t> decompressInt(const rust::Vec<uint8_t>& input, size_t inputElements) {
    std::vector<char> input_vec(input.begin(), input.end());
    std::vector<int64_t> decompressed_data(inputElements);
    ALG_CLASS<int64_t>::decompress(input_vec, inputElements, decompressed_data);

    rust::Vec<int64_t> result;
    result.reserve(decompressed_data.size());
    for (int64_t value : decompressed_data) {
        result.push_back(value);
    }
    return result;
}

rust::Vec<double> decompressDouble(const rust::Vec<uint8_t>& input, size_t inputElements) {
    std::vector<char> input_vec(input.begin(), input.end());
    std::vector<double> decompressed_data(inputElements);
    ALG_CLASS<double>::decompress(input_vec, inputElements, decompressed_data);

    rust::Vec<double> result;
    result.reserve(decompressed_data.size());
    for (double value : decompressed_data) {
        result.push_back(value);
    }
    return result;
}

size_t maxCompressedSize(size_t count) {
	return ALG_CLASS<double>::maxCompressedSize(count);
}

}  // end namespace middleout

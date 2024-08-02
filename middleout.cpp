/*

Copyright (c) 2017, Schizofreny s.r.o - info@schizofreny.com
All rights reserved.

See LICENSE.md file

*/

#include <vector>
#include <stdint.h>
#include <stdlib.h>
#include <memory>
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
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

std::vector<char> compressInt(std::vector<int64_t>& data) {
	std::vector<char> output(middleout::maxCompressedSize(data.size()));
	size_t size = ALG_CLASS<int64_t>::compress(data, output);
	output.resize(size);
	output.shrink_to_fit();	
	return output;
}

std::vector<char> compressDouble(std::vector<double>& data) {
	std::vector<char> output(middleout::maxCompressedSize(data.size()));
	size_t size = ALG_CLASS<double>::compress(data, output);
	output.resize(size);
	output.shrink_to_fit();	
	return output;
}

std::vector<int64_t> decompressInt(std::vector<char>& input, size_t inputElements) {
	std::vector<int64_t> data(inputElements);
	ALG_CLASS<int64_t>::decompress(input, inputElements, data);
	return data;
}

std::vector<double> decompressDouble(std::vector<char>& input, size_t inputElements) {
	std::vector<double> data(inputElements);
	ALG_CLASS<double>::decompress(input, inputElements, data);
	return data;
}

size_t maxCompressedSize(size_t count) {
	return ALG_CLASS<double>::maxCompressedSize(count);
}

}  // end namespace middleout

namespace py = pybind11;

PYBIND11_MODULE(middleout, m) {
    m.doc() = "pybind11 middleout plugin"; // optional module docstring

    m.def("compressInt", &middleout::compressInt);
    m.def("compressDouble", &middleout::compressDouble);

    m.def("decompressInt", &middleout::decompressInt);
    m.def("decompressDouble", &middleout::decompressDouble);

    m.def("maxCompressedSize", &middleout::maxCompressedSize,
          "Calculate max compressed size for given number of elements");
}

// Copyright 2024 Yoshiteru Nagata All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod scalar;

use scalar::Scalar;
// type alias: 現在は Scalar を採用
type AlgClass<T> = Scalar<T>;

/// i64 用の圧縮処理  
/// 入力データを Vec<i64> からコピーして圧縮し、結果を Vec<u8> として返します。
pub fn middleout_compress_int(data: &[i64]) -> Vec<u8> {
    let mut data_vec = data.to_vec();
    let alg = AlgClass::<i64>::new();
    alg.compress_simple(&mut data_vec)
}

/// f64 用の圧縮処理
pub fn middleout_compress_double(data: &[f64]) -> Vec<u8> {
    let mut data_vec = data.to_vec();
    let alg = AlgClass::<f64>::new();
    alg.compress_simple(&mut data_vec)
}

/// i64 用の伸長処理  
/// 圧縮データ（u8 のスライス）と伸長後の要素数を指定して、Vec<i64> を返します。
pub fn middleout_decompress_int(input: &[u8], input_elements: usize) -> Vec<i64> {
    let alg = AlgClass::<i64>::new();
    let mut decompressed_data = vec![0i64; input_elements];
    alg.decompress(input, input_elements, &mut decompressed_data);
    decompressed_data
}

/// f64 用の伸長処理
pub fn middleout_decompress_double(input: &[u8], input_elements: usize) -> Vec<f64> {
    let alg = AlgClass::<f64>::new();
    let mut decompressed_data = vec![0f64; input_elements];
    alg.decompress(input, input_elements, &mut decompressed_data);
    decompressed_data
}

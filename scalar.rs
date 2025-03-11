use std::mem;
use std::ptr;
use std::slice;

use log::debug;


// VECTOR_SIZE は 8 とする
const VECTOR_SIZE: usize = 8;
// 圧縮・伸長対象とする最小データサイズ（適宜調整）
const MIN_DATA_SIZE_COMPRESSION_THRESHOLD: usize = 16;

/// 補助関数：与えられた n のうち 8 の倍数に丸めた値を返す
#[inline]
fn floor8(n: u32) -> u32 {
    n & !7
}

/// 補助関数：オフセットヘッダ部のバイト長を計算する（例： n*3 ビットを丸めてバイト単位）
#[inline]
fn get_bytes_length_of_offsets(n: u32) -> usize {
    ((n * 3 + 7) >> 3) as usize
}

/// 補助関数：初期リファレンス値を出力バッファにコピーする
/// （data から各ブロック先頭の値を output に書き込む）
fn fill_start<T: Copy>(data: &[T], output: &mut [u8], block_size: usize) {
    // 各 VECTOR_SIZE 個のブロックの先頭値を書き込む
    let t_size = mem::size_of::<T>();
    for j in 0..VECTOR_SIZE {
        let data_index = block_size * j;
        let val = data[data_index];
        // 出力先のオフセット
        let out_index = j * t_size;
        unsafe {
            let dst = output[out_index..].as_mut_ptr() as *mut T;
            ptr::write_unaligned(dst, val);
        }
    }
}

/// 圧縮対象がしきい値以下の場合の処理（ここでは単に data のバイト列を output にコピー）
fn do_not_compress_the_data<T: Copy>(data: &[T], output: &mut [u8]) -> usize {
    let t_size = mem::size_of::<T>();
    let byte_len = data.len() * t_size;
    unsafe {
        let data_bytes = slice::from_raw_parts(data.as_ptr() as *const u8, byte_len);
        output[..byte_len].copy_from_slice(data_bytes);
    }
    byte_len
}

/// 伸長対象がしきい値以下の場合の処理（ここでは単に input のバイト列を data にコピー）
fn do_not_decompress_the_data<T: Copy>(input: &[u8], input_elements: usize, data: &mut [T]) {
    unsafe {
        let in_ptr = input.as_ptr() as *const T;
        let in_slice = slice::from_raw_parts(in_ptr, input_elements);
        data[..].copy_from_slice(in_slice);
    }
}

/// Scalar 構造体。型 T は 8 バイト（f64, i64, u64 等）であることを前提。
pub struct Scalar<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Scalar<T>
where
    T: Copy + Default + std::fmt::Debug,
{
    pub fn new() -> Self {
        Scalar {
            _marker: std::marker::PhantomData,
        }
    }

    /// シンプルな圧縮関数。圧縮結果は Vec<u8> として返す。
    pub fn compress_simple(&self, data: &mut Vec<T>) -> Vec<u8> {
        // max_compressed_size() はデータサイズに依存した予想バッファサイズ（適宜実装してください）
        let max_size = Self::max_compressed_size(data.len());
        let mut compressed = vec![0u8; max_size];
        let size = Self::compress(&self, data, &mut compressed);
        compressed.truncate(size);
        compressed.shrink_to_fit();
        compressed
    }

    /// 仮の max_compressed_size 実装（必要に応じて調整してください）
    pub fn max_compressed_size(input_len: usize) -> usize {
        // とりあえず入力サイズの 2 倍分確保する例
        input_len * mem::size_of::<T>() * 2
    }

    /// メインの圧縮処理
    pub fn compress(&self, data: &mut [T], output: &mut [u8]) -> usize {
        if data.len() <= MIN_DATA_SIZE_COMPRESSION_THRESHOLD {
            return do_not_compress_the_data(data, output);
        }

        let t_size = mem::size_of::<T>();
        // 出力バッファの先頭 VECTOR_SIZE 個の T 用領域は初期値用として予約
        let mut output_index = VECTOR_SIZE * mem::size_of::<i64>(); // ※ i64 と T のサイズは同じ前提

        // 中間ブロックサイズ（各ブロックの要素数）
        let block_size = data.len() / VECTOR_SIZE;
        // 初期リファレンス値をコピー
        fill_start(data, output, block_size);

        // メイン圧縮ループ
        for i in 1..block_size {
            let mut same_mask: u8 = 0;
            let mut max_length: i32 = 0;
            let mut compressed_offsets: u32 = 0;
            let mut xored_shifted = [0u64; VECTOR_SIZE];
            let mut data_store_flags = [0; VECTOR_SIZE];
            let mut offsets_shift: u32 = 3; // 最初の 3 ビットは max_length 用
            let mut not_same_count = 0;

            for j in 0..VECTOR_SIZE {
                let offset = block_size * j + i;
                // C++ の reinterpret_cast 相当：T を u64 として読み込む（T は 8 バイトであることが前提）
                let prev = unsafe { mem::transmute_copy::<T, u64>(&data[offset - 1]) };
                let curr = unsafe { mem::transmute_copy::<T, u64>(&data[offset]) };
                let xored = prev ^ curr;

                if xored == 0 {
                    same_mask |= 1 << j;
                    continue;
                }
                data_store_flags[j] = 1;
                let leading_zeros = xored.leading_zeros();
                let trailing_zeros = xored.trailing_zeros();

                let right_offset_bits = floor8(trailing_zeros) as u32;
                let right_offset_bytes = trailing_zeros >> 3;

                let _leading_zeros_bytes = leading_zeros >> 3;
                let _trailing_zeros_bytes = trailing_zeros >> 3;
                let length_bytes = 8 - (_leading_zeros_bytes + _trailing_zeros_bytes);
                max_length = max_length.max(length_bytes as i32);

                compressed_offsets |= (right_offset_bytes as u32) << offsets_shift;
                offsets_shift += 3;
                not_same_count += 1;

                xored_shifted[j] = xored >> right_offset_bits;
            }

            // 書き込み：まず same_mask を 1 バイト出力
            output[output_index] = same_mask;
            output_index += 1;

            // すべて同じなら、以降のメタデータは不要
            if same_mask == 0xFF {
                continue;
            }

            // 4 バイト分のヘッダ：compressed_offsets と (max_length - 1) を格納
            let header: u32 = compressed_offsets | ((max_length - 1) as u32);
            let header_bytes = header.to_le_bytes();
            output[output_index..output_index + 4].copy_from_slice(&header_bytes);

            // オフセットヘッダ部のサイズ分スキップ
            output_index += get_bytes_length_of_offsets((not_same_count + 1) as u32);

            // 各ブロックの値を書き込む
            for j in 0..VECTOR_SIZE {
                // xored_shifted[j] を T に変換して書き込む
                let val: T = unsafe { mem::transmute_copy::<u64, T>(&xored_shifted[j]) };
                unsafe {
                    let dst = output[output_index..].as_mut_ptr() as *mut T;
                    ptr::write_unaligned(dst, val);
                }
                output_index += data_store_flags[j] as usize * (max_length as usize);
            }
        }

        // 圧縮されなかった残りのデータをそのまま書き込む
        for i in (block_size * VECTOR_SIZE)..data.len() {
            unsafe {
                let dst = output[output_index..].as_mut_ptr() as *mut T;
                ptr::write_unaligned(dst, data[i]);
            }
            output_index += t_size;
        }

        // 圧縮アルゴリズムのバージョン／定数として 0x7E を書く
        output[output_index] = 0x7E;
        output_index += 1;

        // 後続のデータアクセスを防ぐために余分な領域を確保（元コードと同様）
        debug!("{:#?} output_index + 6: {}", data[0], output_index + 6);
        output_index + 6
    }

    /// データ伸長（decompression）
    pub fn decompress(&self, input: &[u8], input_elements: usize, data: &mut [T]) {
        if input_elements <= MIN_DATA_SIZE_COMPRESSION_THRESHOLD {
            do_not_decompress_the_data(input, input_elements, data);
            return;
        }

        let t_size = mem::size_of::<T>();
        let block_size = input_elements / VECTOR_SIZE;

        // 最初の VECTOR_SIZE 個のリファレンス値を input から data にコピーする
        // input の先頭部は T 型の値として格納されていると仮定
        for i in 0..VECTOR_SIZE {
            let in_offset = i * t_size;
            let val = unsafe {
                // アラインメントに注意して unaligned な読み出し
                ptr::read_unaligned(input[in_offset..].as_ptr() as *const T)
            };
            data[block_size * i] = val;
        }

        let mut input_index = VECTOR_SIZE * mem::size_of::<i64>(); // i64 と T のサイズは同じ前提
        let mut block_index = 1;
        while block_index < block_size.saturating_sub(5) {
            decompress_block::<false, T>(input, data, &mut input_index, block_size, block_index);
            block_index += 1;
        }
        while block_index < block_size {
            decompress_block::<true, T>(input, data, &mut input_index, block_size, block_index);
            block_index += 1;
        }

        // 圧縮されなかった残りのデータをコピー
        for i in (block_size * VECTOR_SIZE)..input_elements {
            let val = unsafe {
                ptr::read_unaligned(input[input_index..].as_ptr() as *const T)
            };
            data[i] = val;
            input_index += t_size;
        }
    }
}

/// decompress_value を Rust 版に移植
#[inline]
fn decompress_value<T: Copy>(
    j: usize,
    block_size: usize,
    input: &[u8],
    data: &mut [T],
    clear_top_bit_mask: u64,
    input_index: &mut usize,
    i: usize,
    offsets_shift: &mut i32,
    max_length: u8,
    compressed_offsets: u32,
    same_mask: u8,
) {
    let offset = block_size * j + i;
    // 前回の値（data[offset - 1]）を u64 として取得
    let prev = unsafe { mem::transmute_copy::<T, u64>(&data[offset - 1]) };

    // 現在のブロックのシフトビット数を算出
    let shift_bits = (((compressed_offsets >> *offsets_shift) & 0b111) * 8) as u32;
    // input から unaligned な u64 値を読み出す
    let to_xor = {
        let bytes = &input[*input_index..*input_index + 8];
        u64::from_le_bytes(bytes.try_into().unwrap())
    } & clear_top_bit_mask;
    let result = prev ^ (to_xor << shift_bits);

    let is_same = same_mask & (1 << j);
    let modified_offsets_shift = *offsets_shift + 3;
    let modified_input_index = *input_index + max_length as usize;

    // 条件付き分岐（元 asm の cmov 相当）
    if is_same == 0 {
        *offsets_shift = modified_offsets_shift;
        *input_index = modified_input_index;
        // result を用いて値を更新
        unsafe {
            let dst = &mut data[offset] as *mut T;
            ptr::write_unaligned(dst, mem::transmute_copy::<u64, T>(&result));
        }
    } else {
        // 変更なしの場合は前回の値をそのまま書き込む
        unsafe {
            let dst = &mut data[offset] as *mut T;
            ptr::write_unaligned(dst, mem::transmute_copy::<u64, T>(&prev));
        }
    }
}

/// decompress_block を Rust 版に移植。CHECK_FOR_ALL_SAME が true の場合、全て同一なら input_index を巻き戻す。
fn decompress_block<const CHECK_FOR_ALL_SAME: bool, T: Copy>(
    input: &[u8],
    data: &mut [T],
    input_index: &mut usize,
    block_size: usize,
    i: usize,
) {
    let same_mask = input[*input_index];
    *input_index += 1;
    let start_input_index = *input_index;
    if CHECK_FOR_ALL_SAME && same_mask == 0xFF {
        for j in 0..VECTOR_SIZE {
            let offset = block_size * j + i;
            // 前の値をコピー
            data[offset] = data[offset - 1];
        }
        return;
    }

    // 4 バイトのヘッダ読み出し
    let compressed_offsets_and_max_length = {
        let bytes = &input[*input_index..*input_index + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    };
    let max_length = ((compressed_offsets_and_max_length & 0b111) + 1) as u8;

    // same_mask の 1 のビット数を数える
    let same_count = same_mask.count_ones();
    *input_index += get_bytes_length_of_offsets((VECTOR_SIZE as u32).saturating_sub(same_count) + 1);

    let mut offsets_shift: i32 = 3;
    let clear_top_bit_mask = !0u64 >> (64 - 8 * (max_length as u32));

    for j in 0..VECTOR_SIZE {
        decompress_value(
            j,
            block_size,
            input,
            data,
            clear_top_bit_mask,
            input_index,
            i,
            &mut offsets_shift,
            max_length,
            compressed_offsets_and_max_length,
            same_mask,
        );
    }

    // 元コードの inline asm 相当：もし same_mask == 0xFF なら input_index を元に戻す
    if same_mask == 0xFF {
        *input_index = start_input_index;
    }
}

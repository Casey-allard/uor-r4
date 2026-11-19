#pragma once
// All file fields are little-endian u32. Bank pairs contain packed u16 roots.
#define ABI_MAGIC 0x32415355u
#define TRACE_MAGIC 0x314f5355u
#define CAPACITY 256
#define RECORD_WORDS 8
#define MAX_TAPE 24
#define MAX_SLOTS 10
#define ROW_HEAD 8
#define TOKENS ROW_HEAD
#define CLASSES (TOKENS + MAX_TAPE)
#define ROOTS (CLASSES + MAX_TAPE)
#define BANK (ROOTS + MAX_TAPE)
#define ROW_WORDS (BANK + CAPACITY * RECORD_WORDS)
#define TRACE_HEAD 8
#define CAND_WORDS 20
#define WRITER (TRACE_HEAD + 3 * CAND_WORDS)
#define TRACE_WORDS (WRITER + 8 * 8)

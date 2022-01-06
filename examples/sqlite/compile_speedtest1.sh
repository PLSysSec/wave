if [ $# -lt 2 ]
    then
    echo "Argument needed: ./compile_speedtest1.sh <path_to_wasi-sdk-14.0> <path_to_sqlite_root> <path_to_stubs>"
else
    pwd
    echo $1
    echo $2
    echo $3
fi

export CC="$1/bin/clang --sysroot $1/share/wasi-sysroot/"
export CXX="$1/bin/clang++ --sysroot $1/share/wasi-sysroot/"
export LD="$1/bin/wasm-ld"
export AR="$1/bin/llvm-ar"
export RANLIB="$1/bin/llvm-ranlib"
export NM="$1/bin/llvm-nm"
export WASI_CC="$1/bin/clang"
export WASI_CXX="$1/bin/clang++"
export WASI_LD="$1/bin/wasm-ld"
export WASI_AR="$1/bin/llvm-ar"
export WASI_RANLIB="$1/bin/llvm-ranlib"
export WASI_NM="$1/bin/llvm-nm"
export CFLAGS="-DSQLITE_OMIT_LOAD_EXTENSION=1 -DSQLITE_OMIT_RANDOMNESS -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_GETPID -I$3"
export LDFLAGS="-lwasi-emulated-signal -lwasi-emulated-mman -lwasi-emulated-getpid"
echo $CFLAGS
$2/configure --host=wasm32-wasi --disable-tcl --disable-threadsafe
make -j8 sqlite3.c
sed -i '1s/^/#include "dummy.h"\n/' sqlite3.c
make -j8 speedtest1
